pub trait AccessManaged {
    const METHODS: &[&'static str];

    fn method(&self) -> &'static str;
}
