// `nsISupports` is a Gecko thing that can be queried if it implements some
// interface. We can query anything via `JsCast`, so it is good enough to just
// call it an Object.
typedef object nsISupports;
