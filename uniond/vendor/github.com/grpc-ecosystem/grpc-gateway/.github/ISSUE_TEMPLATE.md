# The gRPC-Gateway project is maintained by volunteers in their spare time. Please follow these troubleshooting steps before submitting an issue.

- [ ] Check if your issue has already been reported (https://github.com/grpc-ecosystem/grpc-gateway/issues).
- [ ] Update your protoc to the [latest version](https://github.com/google/protobuf/releases).
- [ ] Update your copy of the `grpc-gateway` library to the latest version from github:
  ```sh
  go get -u github.com/grpc-ecosystem/grpc-gateway
  ```
- [ ] Delete the `protoc-gen-grpc-gateway` and `protoc-gen-swagger` binary from your `PATH`,
  and reinstall the latest versions:
  ```sh
  go get -u github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway
  go get -u github.com/grpc-ecosystem/grpc-gateway/protoc-gen-swagger
  ```
  
## I still have a problem!
 
Please consider reaching out for help on a chat forum, such as
[Gophers Slack](https://invite.slack.golangbridge.org/) (channel #grpc-gateway).
It's much easier to help with common debugging steps in a chat, and some of
the maintainers are reading the channel regularly. If you
submit an issue which is clearly an environment setup problem, or it's obvious
you haven't tried seeking help somewhere else first, we may close your issue.
 
## I still have a problem!

Please follow these steps to submit a bug report:

### Bug reports:

Fill in the following sections with explanations of what's gone wrong.

### Steps you follow to reproduce the error:

```html
<!-- Example steps
1.  I grab my catapult
2.  I load it with lettuce
3.  Press the fire button
4.  It falls over
-->
```

Your steps here.

### What did you expect to happen instead:

```html
<!-- Example answer
1.  It would have rained lettuce from the sky bringing forth a cuddly army of bunnies we could
    play with
-->
```

Your answer here.

### What's your theory on why it isn't working:

```html
<!-- Example answer
Evil wizards are hoarding the bunnies and don't want to share. The wizards are casting 
lettuce protection spells so the catapult won't work.
-->
```

Your theory here.
