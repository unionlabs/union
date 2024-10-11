package main

import (
	"fmt"
	"log"

	"github.com/aymanbagabas/go-osc52"
	"github.com/charmbracelet/wish"
	"github.com/gliderlabs/ssh"
)

func main() {
	s, err := wish.NewServer(
		wish.WithAddress(":2222"),
		wish.WithHostKeyPath("ssh_host_key"),
		wish.WithMiddleware(
			middleware(),
		),
	)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("SSH into %s\n", s.Addr)
	s.ListenAndServe()
}

func middleware() wish.Middleware {
	return func(h ssh.Handler) ssh.Handler {
		return func(s ssh.Session) {
			environ := s.Environ()
			pty, _, _ := s.Pty()
			// Put TERM environment variable into environ.
			environ = append(environ, fmt.Sprintf("TERM=%s", pty.Term))
			out := osc52.NewOutput(s, environ)
			str := "hello world"
			out.Copy(str)
			s.Write([]byte(fmt.Sprintf("Copied %q!\n", str)))
		}
	}
}
