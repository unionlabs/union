export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

// export const load: LayoutLoad = async ({ url }) => {
//   const pathname = url.pathname
//
//   console.log('load')
//
//   if (pathname) {
//     const segments = pathname.split("/").filter(Boolean)
//     if (segments[0] === "terminal") {
//       const authCheck = await checkAuth()
//       return authCheck.match(
//         () => {
//           return {}
//         },
//         (error: SessionError) => {
//           console.error(error.message)
//           throw redirect(302, "/")
//         }
//       )
//     }
//   }
//
//   return {}
// }
