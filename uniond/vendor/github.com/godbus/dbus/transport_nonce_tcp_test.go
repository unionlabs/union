package dbus

import (
	"bufio"
	"io/ioutil"
	"os"
	"os/exec"
	"testing"
)

func TestTcpNonceConnection(t *testing.T) {
	addr, process := startDaemon(t, `<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
	<type>session</type>
		<listen>nonce-tcp:</listen>
		<auth>ANONYMOUS</auth>
		<allow_anonymous/>
                <apparmor mode="disabled"/>
		<policy context="default">
			<allow send_destination="*" eavesdrop="true"/>
			<allow eavesdrop="true"/>
			<allow own="*"/>
		</policy>
</busconfig>
`)
	defer process.Kill()

	c, err := Dial(addr)
	if err != nil {
		t.Fatal(err)
	}
	if err = c.Auth([]Auth{AuthAnonymous()}); err != nil {
		t.Fatal(err)
	}
	if err = c.Hello(); err != nil {
		t.Fatal(err)
	}
}

// startDaemon starts a dbus-daemon instance with the given config
// and returns its address string and underlying process.
func startDaemon(t *testing.T, config string) (string, *os.Process) {
	cfg, err := ioutil.TempFile("", "")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(cfg.Name())
	if _, err = cfg.Write([]byte(config)); err != nil {
		t.Fatal(err)
	}

	cmd := exec.Command("dbus-daemon", "--nofork", "--print-address", "--config-file", cfg.Name())
	cmd.Stderr = os.Stderr
	out, err := cmd.StdoutPipe()
	if err != nil {
		t.Fatal(err)
	}
	if err = cmd.Start(); err != nil {
		t.Fatal(err)
	}
	r := bufio.NewReader(out)
	l, _, err := r.ReadLine()
	if err != nil {
		cmd.Process.Kill()
		t.Fatal(err)
	}
	return string(l), cmd.Process
}
