<!doctype html>
<html lang='en'>

<head>
  <meta charset='UTF-8' />
  <title>{{ .Title }}</title>
  <link rel='stylesheet' type='text/css' href='//unpkg.com/swagger-ui-dist@5.13.0/swagger-ui.css' />
  <style>
    *,
    *:before,
    *:after {
      box-sizing: inherit;
    }

    *::-webkit-scrollbar {
      height: 0.3rem;
      width: 0rem;
    }

    *::-webkit-scrollbar-track {
      -ms-overflow-style: none;
      overflow: -moz-scrollbars-none;
    }

    *::-webkit-scrollbar-thumb {
      -ms-overflow-style: none;
      overflow: -moz-scrollbars-none;
    }

    @supports (scrollbar-gutter: stable) {
      html {
        overflow-y: auto;
        scrollbar-gutter: stable;
      }
    }

    html {
      box-sizing: border-box;
      overflow: -moz-scrollbars-vertical;
      overflow-y: scroll;
      margin: 0;
      background: #FFFFFF;
    }

    .errors-wrapper {
      display: none !important;
    }
  </style>
</head>

<body>
  <main id='swagger-ui'>
    <script src='//unpkg.com/swagger-ui-dist@5.13.0/swagger-ui-bundle.js' charset='UTF-8'></script>
    <script src='//unpkg.com/swagger-ui-dist@5.13.0/swagger-ui-standalone-preset.js' charset='UTF-8'></script>

    <script>
      window.onload = () => window.ui =
        SwaggerUIBundle({
          url: {{ .URL }},
          deepLinking: true,
          dom_id: '#swagger-ui',
          layout: 'StandaloneLayout',
          plugins: [SwaggerUIBundle.plugins.DownloadUrl],
          presets: [SwaggerUIBundle.presets.apis, SwaggerUIStandalonePreset],
        })
    </script>
</body>

</html>

