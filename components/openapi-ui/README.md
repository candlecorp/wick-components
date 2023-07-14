# openapi-ui

Example config:

```yaml
---
kind: wick/app@v1
name: rest_api_app
resources:
  - name: http
    resource:
      kind: wick/resource/tcpport@v1
      port: '{{ ctx.env.HTTP_PORT | default: 8999 }}'
      address: 0.0.0.0
import:
  - name: sample
    component:
      kind: wick/component/manifest@v1
      ref: ./rest-router/component.wick
  - name: openapi
    component:
      kind: wick/component/manifest@v1
      ref: registry.candle.dev/common/openapi-ui:0.3.0
      with:
        schema_url: /openapi.json
triggers:
  - kind: wick/trigger/http@v1
    resource: http
    routers:
      - kind: wick/router/raw@v1
        path: /openapi-ui
        codec: Raw
        operation: openapi::serve
      - kind: wick/router/rest@v1
        path: /
        tools:
          openapi: true
        info:
          title: 'Sample REST API'
          description: 'A sample REST API'
          version: '0.0.1'
        routes:
          - sub_path: '/this/{first:string}/some/{second:u32}?third:string[]&fourth:bool'
            operation: sample::echo
            description: 'Echoes inputs first, second, third, and fourth back as JSON'
```