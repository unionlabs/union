# \ResponseBodyServiceApi

All URIs are relative to *https://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ResponseBodyServiceGetResponseBody**](ResponseBodyServiceApi.md#ResponseBodyServiceGetResponseBody) | **Get** /responsebody/{data} | 
[**ResponseBodyServiceGetResponseBodyStream**](ResponseBodyServiceApi.md#ResponseBodyServiceGetResponseBodyStream) | **Get** /responsebody/stream/{data} | 
[**ResponseBodyServiceListResponseBodies**](ResponseBodyServiceApi.md#ResponseBodyServiceListResponseBodies) | **Get** /responsebodies/{data} | 
[**ResponseBodyServiceListResponseStrings**](ResponseBodyServiceApi.md#ResponseBodyServiceListResponseStrings) | **Get** /responsestrings/{data} | 


# **ResponseBodyServiceGetResponseBody**
> ExamplepbResponseBodyOutResponse ResponseBodyServiceGetResponseBody(ctx, data)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
  **data** | **string**|  | 

### Return type

[**ExamplepbResponseBodyOutResponse**](examplepbResponseBodyOutResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ResponseBodyServiceGetResponseBodyStream**
> StreamResultOfExamplepbResponseBodyOut ResponseBodyServiceGetResponseBodyStream(ctx, data)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
  **data** | **string**|  | 

### Return type

[**StreamResultOfExamplepbResponseBodyOut**](Stream result of examplepbResponseBodyOut.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ResponseBodyServiceListResponseBodies**
> []ExamplepbRepeatedResponseBodyOutResponse ResponseBodyServiceListResponseBodies(ctx, data)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
  **data** | **string**|  | 

### Return type

[**[]ExamplepbRepeatedResponseBodyOutResponse**](examplepbRepeatedResponseBodyOutResponse.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ResponseBodyServiceListResponseStrings**
> []string ResponseBodyServiceListResponseStrings(ctx, data)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context for authentication, logging, cancellation, deadlines, tracing, etc.
  **data** | **string**|  | 

### Return type

**[]string**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

