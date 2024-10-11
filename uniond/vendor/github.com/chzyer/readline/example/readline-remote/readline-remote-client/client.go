package main

import "github.com/chzyer/readline"

func main() {
	if err := readline.DialRemote("tcp", ":12344"); err != nil {
		println(err.Error())
	}
}
