## Reference:
   CBOR Encoding is described in [RFC7049](https://tools.ietf.org/html/rfc7049)

## Comparison of JSON vs CBOR

Two main areas of reduction are:

1. CPU usage to write a log msg 
2. Size (in bytes) of log messages.


CPU Usage savings are below:
```
name                                    JSON time/op    CBOR time/op   delta
Info-32                                   15.3ns ± 1%    11.7ns ± 3%  -23.78%  (p=0.000 n=9+10)      
ContextFields-32                          16.2ns ± 2%    12.3ns ± 3%  -23.97%  (p=0.000 n=9+9)       
ContextAppend-32                          6.70ns ± 0%    6.20ns ± 0%   -7.44%  (p=0.000 n=9+9)       
LogFields-32                              66.4ns ± 0%    24.6ns ± 2%  -62.89%  (p=0.000 n=10+9)      
LogArrayObject-32                          911ns ±11%     768ns ± 6%  -15.64%  (p=0.000 n=10+10)     
LogFieldType/Floats-32                    70.3ns ± 2%    29.5ns ± 1%  -57.98%  (p=0.000 n=10+10)     
LogFieldType/Err-32                       14.0ns ± 3%    12.1ns ± 8%  -13.20%  (p=0.000 n=8+10)      
LogFieldType/Dur-32                       17.2ns ± 2%    13.1ns ± 1%  -24.27%  (p=0.000 n=10+9)      
LogFieldType/Object-32                    54.3ns ±11%    52.3ns ± 7%     ~     (p=0.239 n=10+10)     
LogFieldType/Ints-32                      20.3ns ± 2%    15.1ns ± 2%  -25.50%  (p=0.000 n=9+10)      
LogFieldType/Interfaces-32                 642ns ±11%     621ns ± 9%     ~     (p=0.118 n=10+10)     
LogFieldType/Interface(Objects)-32         635ns ±13%     632ns ± 9%     ~     (p=0.592 n=10+10)     
LogFieldType/Times-32                      294ns ± 0%      27ns ± 1%  -90.71%  (p=0.000 n=10+9)      
LogFieldType/Durs-32                       121ns ± 0%      33ns ± 2%  -72.44%  (p=0.000 n=9+9)       
LogFieldType/Interface(Object)-32         56.6ns ± 8%    52.3ns ± 8%   -7.54%  (p=0.007 n=10+10)     
LogFieldType/Errs-32                      17.8ns ± 3%    16.1ns ± 2%   -9.71%  (p=0.000 n=10+9)      
LogFieldType/Time-32                      40.5ns ± 1%    12.7ns ± 6%  -68.66%  (p=0.000 n=8+9)       
LogFieldType/Bool-32                      12.0ns ± 5%    10.2ns ± 2%  -15.18%  (p=0.000 n=10+8)      
LogFieldType/Bools-32                     17.2ns ± 2%    12.6ns ± 4%  -26.63%  (p=0.000 n=10+10)     
LogFieldType/Int-32                       12.3ns ± 2%    11.2ns ± 4%   -9.27%  (p=0.000 n=9+10)      
LogFieldType/Float-32                     16.7ns ± 1%    12.6ns ± 2%  -24.42%  (p=0.000 n=7+9)       
LogFieldType/Str-32                       12.7ns ± 7%    11.3ns ± 7%  -10.88%  (p=0.000 n=10+9)      
LogFieldType/Strs-32                      20.3ns ± 3%    18.2ns ± 3%  -10.25%  (p=0.000 n=9+10)      
LogFieldType/Interface-32                  183ns ±12%     175ns ± 9%     ~     (p=0.078 n=10+10)     
```

Log message size savings is greatly dependent on the number and type of fields in the log message.
Assuming this log message (with an Integer, timestamp and string, in addition to level).

`{"level":"error","Fault":41650,"time":"2018-04-01T15:18:19-07:00","message":"Some Message"}`

Two measurements were done for the log file sizes - one without any compression, second
using [compress/zlib](https://golang.org/pkg/compress/zlib/). 

Results for 10,000 log messages:

| Log Format |  Plain File Size (in KB) | Compressed File Size (in KB) |
| :--- | :---: | :---: |
| JSON | 920 | 28 |
| CBOR | 550 | 28 |

The example used to calculate the above data is available in [Examples](examples).
