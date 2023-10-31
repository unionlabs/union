use *;

extern "C" {
    pub fn NCONF_new(meth: *mut CONF_METHOD) -> *mut CONF;
    pub fn NCONF_default() -> *mut CONF_METHOD;
    pub fn NCONF_free(conf: *mut CONF);
}
