exports.is_shared_array_buffer_supported = function () {
    return typeof SharedArrayBuffer === 'function';
};
