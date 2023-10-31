//! Routing between [`Service`]s and handlers.

use self::{future::RouteFuture, not_found::NotFound};
use crate::{
    body::{boxed, Body, Bytes, HttpBody},
    extract::connect_info::IntoMakeServiceWithConnectInfo,
    response::{IntoResponse, Redirect, Response},
    routing::strip_prefix::StripPrefix,
    util::try_downcast,
    BoxError,
};
use http::{Request, Uri};
use matchit::MatchError;
use std::{
    borrow::Cow,
    collections::HashMap,
    convert::Infallible,
    fmt,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{layer::layer_fn, ServiceBuilder};
use tower_http::map_response_body::MapResponseBodyLayer;
use tower_layer::Layer;
use tower_service::Service;

pub mod future;

mod into_make_service;
mod method_filter;
mod method_routing;
mod not_found;
mod route;
mod strip_prefix;
pub(crate) mod url_params;

#[cfg(test)]
mod tests;

pub use self::{into_make_service::IntoMakeService, method_filter::MethodFilter, route::Route};

pub use self::method_routing::{
    any, any_service, delete, delete_service, get, get_service, head, head_service, on, on_service,
    options, options_service, patch, patch_service, post, post_service, put, put_service, trace,
    trace_service, MethodRouter,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RouteId(u32);

impl RouteId {
    fn next() -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        // `AtomicU64` isn't supported on all platforms
        static ID: AtomicU32 = AtomicU32::new(0);
        let id = ID.fetch_add(1, Ordering::Relaxed);
        if id == u32::MAX {
            panic!("Over `u32::MAX` routes created. If you need this, please file an issue.");
        }
        Self(id)
    }
}

/// The router type for composing handlers and services.
pub struct Router<B = Body> {
    routes: HashMap<RouteId, Endpoint<B>>,
    node: Arc<Node>,
    fallback: Fallback<B>,
    nested_at_root: bool,
}

impl<B> Clone for Router<B> {
    fn clone(&self) -> Self {
        Self {
            routes: self.routes.clone(),
            node: Arc::clone(&self.node),
            fallback: self.fallback.clone(),
            nested_at_root: self.nested_at_root,
        }
    }
}

impl<B> Default for Router<B>
where
    B: HttpBody + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<B> fmt::Debug for Router<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Router")
            .field("routes", &self.routes)
            .field("node", &self.node)
            .field("fallback", &self.fallback)
            .field("nested_at_root", &self.nested_at_root)
            .finish()
    }
}

pub(crate) const NEST_TAIL_PARAM: &str = "__private__axum_nest_tail_param";
const NEST_TAIL_PARAM_CAPTURE: &str = "/*__private__axum_nest_tail_param";

impl<B> Router<B>
where
    B: HttpBody + Send + 'static,
{
    /// Create a new `Router`.
    ///
    /// Unless you add additional routes this will respond with `404 Not Found` to
    /// all requests.
    pub fn new() -> Self {
        Self {
            routes: Default::default(),
            node: Default::default(),
            fallback: Fallback::Default(Route::new(NotFound)),
            nested_at_root: false,
        }
    }

    #[doc = include_str!("../docs/routing/route.md")]
    #[track_caller]
    pub fn route<T>(mut self, path: &str, service: T) -> Self
    where
        T: Service<Request<B>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        T::Future: Send + 'static,
    {
        fn validate_path(path: &str) {
            if path.is_empty() {
                panic!("Paths must start with a `/`. Use \"/\" for root routes");
            } else if !path.starts_with('/') {
                panic!("Paths must start with a `/`");
            }
        }

        validate_path(path);

        let service = match try_downcast::<Router<B>, _>(service) {
            Ok(_) => {
                panic!("Invalid route: `Router::route` cannot be used with `Router`s. Use `Router::nest` instead")
            }
            Err(svc) => svc,
        };

        let id = RouteId::next();

        let service = match try_downcast::<MethodRouter<B, Infallible>, _>(service) {
            Ok(method_router) => {
                if let Some((route_id, Endpoint::MethodRouter(prev_method_router))) = self
                    .node
                    .path_to_route_id
                    .get(path)
                    .and_then(|route_id| self.routes.get(route_id).map(|svc| (*route_id, svc)))
                {
                    // if we're adding a new `MethodRouter` to a route that already has one just
                    // merge them. This makes `.route("/", get(_)).route("/", post(_))` work
                    let service =
                        Endpoint::MethodRouter(prev_method_router.clone().merge(method_router));
                    self.routes.insert(route_id, service);
                    return self;
                } else {
                    Endpoint::MethodRouter(method_router)
                }
            }
            Err(service) => Endpoint::Route(Route::new(service)),
        };

        self.set_node(path, id);

        self.routes.insert(id, service);

        self
    }

    #[track_caller]
    fn set_node(&mut self, path: &str, id: RouteId) {
        let mut node =
            Arc::try_unwrap(Arc::clone(&self.node)).unwrap_or_else(|node| (*node).clone());
        if let Err(err) = node.insert(path, id) {
            self.panic_on_matchit_error(err);
        }
        self.node = Arc::new(node);
    }

    #[doc = include_str!("../docs/routing/nest.md")]
    #[track_caller]
    pub fn nest<T>(mut self, mut path: &str, svc: T) -> Self
    where
        T: Service<Request<B>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        T::Future: Send + 'static,
    {
        if path.is_empty() {
            // nesting at `""` and `"/"` should mean the same thing
            path = "/";
        }

        if path.contains('*') {
            panic!("Invalid route: nested routes cannot contain wildcards (*)");
        }

        let prefix = path;

        if path == "/" {
            self.nested_at_root = true;
        }

        match try_downcast::<Router<B>, _>(svc) {
            // if the user is nesting a `Router` we can implement nesting
            // by simplying copying all the routes and adding the prefix in
            // front
            Ok(router) => {
                let Router {
                    mut routes,
                    node,
                    fallback,
                    // nesting a router that has something nested at root
                    // doesn't mean something is nested at root in _this_ router
                    // thus we don't need to propagate that
                    nested_at_root: _,
                } = router;

                if let Fallback::Custom(_) = fallback {
                    panic!("Cannot nest `Router`s that has a fallback");
                }

                for (id, nested_path) in &node.route_id_to_path {
                    let route = routes.remove(id).unwrap();
                    let full_path: Cow<str> = if &**nested_path == "/" {
                        path.into()
                    } else if path == "/" {
                        (&**nested_path).into()
                    } else if let Some(path) = path.strip_suffix('/') {
                        format!("{}{}", path, nested_path).into()
                    } else {
                        format!("{}{}", path, nested_path).into()
                    };
                    self = match route {
                        Endpoint::MethodRouter(method_router) => self.route(
                            &full_path,
                            method_router.layer(layer_fn(|s| StripPrefix::new(s, prefix))),
                        ),
                        Endpoint::Route(route) => {
                            self.route(&full_path, StripPrefix::new(route, prefix))
                        }
                    };
                }

                debug_assert!(routes.is_empty());
            }
            // otherwise we add a wildcard route to the service
            Err(svc) => {
                let path = if path.ends_with('/') {
                    format!("{}*{}", path, NEST_TAIL_PARAM)
                } else {
                    format!("{}/*{}", path, NEST_TAIL_PARAM)
                };

                self = self.route(&path, strip_prefix::StripPrefix::new(svc, prefix));
            }
        }

        self
    }

    #[doc = include_str!("../docs/routing/merge.md")]
    #[track_caller]
    pub fn merge<R>(mut self, other: R) -> Self
    where
        R: Into<Router<B>>,
    {
        let Router {
            routes,
            node,
            fallback,
            nested_at_root,
        } = other.into();

        for (id, route) in routes {
            let path = node
                .route_id_to_path
                .get(&id)
                .expect("no path for route id. This is a bug in axum. Please file an issue");
            self = match route {
                Endpoint::MethodRouter(route) => self.route(path, route),
                Endpoint::Route(route) => self.route(path, route),
            };
        }

        self.fallback = match (self.fallback, fallback) {
            (Fallback::Default(_), pick @ Fallback::Default(_)) => pick,
            (Fallback::Default(_), pick @ Fallback::Custom(_)) => pick,
            (pick @ Fallback::Custom(_), Fallback::Default(_)) => pick,
            (Fallback::Custom(_), Fallback::Custom(_)) => {
                panic!("Cannot merge two `Router`s that both have a fallback")
            }
        };

        self.nested_at_root = self.nested_at_root || nested_at_root;

        self
    }

    #[doc = include_str!("../docs/routing/layer.md")]
    pub fn layer<L, NewReqBody, NewResBody>(self, layer: L) -> Router<NewReqBody>
    where
        L: Layer<Route<B>>,
        L::Service: Service<Request<NewReqBody>, Response = Response<NewResBody>, Error = Infallible>
            + Clone
            + Send
            + 'static,
        <L::Service as Service<Request<NewReqBody>>>::Future: Send + 'static,
        NewResBody: HttpBody<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
    {
        let layer = ServiceBuilder::new()
            .layer(MapResponseBodyLayer::new(boxed))
            .layer(layer)
            .into_inner();

        let routes = self
            .routes
            .into_iter()
            .map(|(id, route)| {
                let route = match route {
                    Endpoint::MethodRouter(method_router) => {
                        Endpoint::MethodRouter(method_router.layer(&layer))
                    }
                    Endpoint::Route(route) => Endpoint::Route(Route::new(layer.layer(route))),
                };
                (id, route)
            })
            .collect();

        let fallback = self.fallback.map(|svc| Route::new(layer.layer(svc)));

        Router {
            routes,
            node: self.node,
            fallback,
            nested_at_root: self.nested_at_root,
        }
    }

    #[doc = include_str!("../docs/routing/route_layer.md")]
    pub fn route_layer<L, NewResBody>(self, layer: L) -> Self
    where
        L: Layer<Route<B>>,
        L::Service: Service<Request<B>, Response = Response<NewResBody>, Error = Infallible>
            + Clone
            + Send
            + 'static,
        <L::Service as Service<Request<B>>>::Future: Send + 'static,
        NewResBody: HttpBody<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
    {
        let layer = ServiceBuilder::new()
            .layer(MapResponseBodyLayer::new(boxed))
            .layer(layer)
            .into_inner();

        let routes = self
            .routes
            .into_iter()
            .map(|(id, route)| {
                let route = match route {
                    Endpoint::MethodRouter(method_router) => {
                        Endpoint::MethodRouter(method_router.layer(&layer))
                    }
                    Endpoint::Route(route) => Endpoint::Route(Route::new(layer.layer(route))),
                };
                (id, route)
            })
            .collect();

        Router {
            routes,
            node: self.node,
            fallback: self.fallback,
            nested_at_root: self.nested_at_root,
        }
    }

    #[doc = include_str!("../docs/routing/fallback.md")]
    pub fn fallback<T>(mut self, svc: T) -> Self
    where
        T: Service<Request<B>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        T::Future: Send + 'static,
    {
        self.fallback = Fallback::Custom(Route::new(svc));
        self
    }

    /// Convert this router into a [`MakeService`], that is a [`Service`] whose
    /// response is another service.
    ///
    /// This is useful when running your application with hyper's
    /// [`Server`](hyper::server::Server):
    ///
    /// ```
    /// use axum::{
    ///     routing::get,
    ///     Router,
    /// };
    ///
    /// let app = Router::new().route("/", get(|| async { "Hi!" }));
    ///
    /// # async {
    /// axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
    ///     .serve(app.into_make_service())
    ///     .await
    ///     .expect("server failed");
    /// # };
    /// ```
    ///
    /// [`MakeService`]: tower::make::MakeService
    pub fn into_make_service(self) -> IntoMakeService<Self> {
        IntoMakeService::new(self)
    }

    #[doc = include_str!("../docs/routing/into_make_service_with_connect_info.md")]
    pub fn into_make_service_with_connect_info<C>(self) -> IntoMakeServiceWithConnectInfo<Self, C> {
        IntoMakeServiceWithConnectInfo::new(self)
    }

    #[inline]
    fn call_route(
        &self,
        match_: matchit::Match<&RouteId>,
        mut req: Request<B>,
    ) -> RouteFuture<B, Infallible> {
        let id = *match_.value;

        #[cfg(feature = "matched-path")]
        {
            fn set_matched_path(
                id: RouteId,
                route_id_to_path: &HashMap<RouteId, Arc<str>>,
                extensions: &mut http::Extensions,
            ) {
                if let Some(matched_path) = route_id_to_path.get(&id) {
                    use crate::extract::MatchedPath;

                    let matched_path = if let Some(previous) = extensions.get::<MatchedPath>() {
                        // a previous `MatchedPath` might exist if we're inside a nested Router
                        let previous = if let Some(previous) =
                            previous.as_str().strip_suffix(NEST_TAIL_PARAM_CAPTURE)
                        {
                            previous
                        } else {
                            previous.as_str()
                        };

                        let matched_path = format!("{}{}", previous, matched_path);
                        matched_path.into()
                    } else {
                        Arc::clone(matched_path)
                    };
                    extensions.insert(MatchedPath(matched_path));
                } else {
                    #[cfg(debug_assertions)]
                    panic!("should always have a matched path for a route id");
                }
            }

            set_matched_path(id, &self.node.route_id_to_path, req.extensions_mut());
        }

        url_params::insert_url_params(req.extensions_mut(), match_.params);

        let mut route = self
            .routes
            .get(&id)
            .expect("no route for id. This is a bug in axum. Please file an issue")
            .clone();

        match &mut route {
            Endpoint::MethodRouter(inner) => inner.call(req),
            Endpoint::Route(inner) => inner.call(req),
        }
    }

    #[track_caller]
    fn panic_on_matchit_error(&self, err: matchit::InsertError) {
        if self.nested_at_root {
            panic!(
                "Invalid route: {}. Note that `nest(\"/\", _)` conflicts with all routes. Use `Router::fallback` instead",
                err,
            );
        } else {
            panic!("Invalid route: {}", err);
        }
    }
}

impl<B> Service<Request<B>> for Router<B>
where
    B: HttpBody + Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = RouteFuture<B, Infallible>;

    #[inline]
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        #[cfg(feature = "original-uri")]
        {
            use crate::extract::OriginalUri;

            if req.extensions().get::<OriginalUri>().is_none() {
                let original_uri = OriginalUri(req.uri().clone());
                req.extensions_mut().insert(original_uri);
            }
        }

        let path = req.uri().path().to_owned();

        match self.node.at(&path) {
            Ok(match_) => self.call_route(match_, req),
            Err(err) => {
                let mut fallback = match &self.fallback {
                    Fallback::Default(inner) => inner.clone(),
                    Fallback::Custom(inner) => inner.clone(),
                };

                let new_uri = match err {
                    MatchError::MissingTrailingSlash => {
                        replace_path(req.uri(), &format!("{}/", &path))
                    }
                    MatchError::ExtraTrailingSlash => {
                        replace_path(req.uri(), path.strip_suffix('/').unwrap())
                    }
                    MatchError::NotFound => None,
                };

                if let Some(new_uri) = new_uri {
                    RouteFuture::from_response(
                        Redirect::permanent(&new_uri.to_string()).into_response(),
                    )
                } else {
                    fallback.call(req)
                }
            }
        }
    }
}

fn replace_path(uri: &Uri, new_path: &str) -> Option<Uri> {
    let mut new_path_and_query = new_path.to_owned();
    if let Some(query) = uri.query() {
        new_path_and_query.push('?');
        new_path_and_query.push_str(query);
    }

    let mut parts = uri.clone().into_parts();
    parts.path_and_query = Some(new_path_and_query.parse().unwrap());

    Uri::from_parts(parts).ok()
}

/// Wrapper around `matchit::Router` that supports merging two `Router`s.
#[derive(Clone, Default)]
struct Node {
    inner: matchit::Router<RouteId>,
    route_id_to_path: HashMap<RouteId, Arc<str>>,
    path_to_route_id: HashMap<Arc<str>, RouteId>,
}

impl Node {
    fn insert(
        &mut self,
        path: impl Into<String>,
        val: RouteId,
    ) -> Result<(), matchit::InsertError> {
        let path = path.into();

        self.inner.insert(&path, val)?;

        let shared_path: Arc<str> = path.into();
        self.route_id_to_path.insert(val, shared_path.clone());
        self.path_to_route_id.insert(shared_path, val);

        Ok(())
    }

    fn at<'n, 'p>(
        &'n self,
        path: &'p str,
    ) -> Result<matchit::Match<'n, 'p, &'n RouteId>, MatchError> {
        self.inner.at(path)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("paths", &self.route_id_to_path)
            .finish()
    }
}

enum Fallback<B, E = Infallible> {
    Default(Route<B, E>),
    Custom(Route<B, E>),
}

impl<B, E> Clone for Fallback<B, E> {
    fn clone(&self) -> Self {
        match self {
            Fallback::Default(inner) => Fallback::Default(inner.clone()),
            Fallback::Custom(inner) => Fallback::Custom(inner.clone()),
        }
    }
}

impl<B, E> fmt::Debug for Fallback<B, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default(inner) => f.debug_tuple("Default").field(inner).finish(),
            Self::Custom(inner) => f.debug_tuple("Custom").field(inner).finish(),
        }
    }
}

impl<B, E> Fallback<B, E> {
    fn map<F, B2, E2>(self, f: F) -> Fallback<B2, E2>
    where
        F: FnOnce(Route<B, E>) -> Route<B2, E2>,
    {
        match self {
            Fallback::Default(inner) => Fallback::Default(f(inner)),
            Fallback::Custom(inner) => Fallback::Custom(f(inner)),
        }
    }
}

enum Endpoint<B> {
    MethodRouter(MethodRouter<B>),
    Route(Route<B>),
}

impl<B> Clone for Endpoint<B> {
    fn clone(&self) -> Self {
        match self {
            Endpoint::MethodRouter(inner) => Endpoint::MethodRouter(inner.clone()),
            Endpoint::Route(inner) => Endpoint::Route(inner.clone()),
        }
    }
}

impl<B> fmt::Debug for Endpoint<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MethodRouter(inner) => inner.fmt(f),
            Self::Route(inner) => inner.fmt(f),
        }
    }
}

#[test]
#[allow(warnings)]
fn traits() {
    use crate::test_helpers::*;
    assert_send::<Router<()>>();
}

// https://github.com/tokio-rs/axum/issues/1122
#[test]
fn test_replace_trailing_slash() {
    let uri = "api.ipify.org:80".parse::<Uri>().unwrap();
    assert!(uri.scheme().is_none());
    assert_eq!(uri.authority(), Some(&"api.ipify.org:80".parse().unwrap()));
    assert!(uri.path_and_query().is_none());

    replace_path(&uri, "/foo");
}
