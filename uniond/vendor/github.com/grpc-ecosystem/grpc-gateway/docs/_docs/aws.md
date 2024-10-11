---
category: documentation
---

# AWS

## Import swagger documentation into AWS API Gateway
The AWS API gateway service allows importing of an OpenAPI specification to create a REST API. The process is very straightforward and can be found [here](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-import-api.html).
Here are some tips to consider when importing the documentation:

1. Remove any circular dependencies (these aren't supported by the parser).
2. Remove security-related annotations (These annotations aren't well supported by the parser).
3. Max length of fields are reviewed by the parser but the errors aren't self-explanatory. Review the [specification](https://swagger.io/specification/v2/) to verify that the requirements are met.
4. API gateway errors aren't great, but you can use this [page](https://apidevtools.org/swagger-parser/online/) for structure validation.
