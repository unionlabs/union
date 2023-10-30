export function new_response() {
  return new Response(null, {status: 501});
}

export function get_wasm_imports() {
  return {
    imports: {
      imported_func: function () {
        return 1;
      }
    }
  };
}
