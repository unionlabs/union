package digitalocean

import (
	"context"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/test/e2e/pkg/exec"
	"github.com/cometbft/cometbft/test/e2e/pkg/infra"
)

var _ infra.Provider = (*Provider)(nil)

// Provider implements a DigitalOcean-backed infrastructure provider.
type Provider struct {
	infra.ProviderData
}

// Noop currently. Setup is performed externally to the e2e test tool.
func (p *Provider) Setup() error {
	for _, n := range p.Testnet.Nodes {
		if n.ClockSkew != 0 {
			return fmt.Errorf("node %q contains clock skew configuration (not supported on DO)", n.Name)
		}
	}

	return nil
}

var ymlPlaybookSeq int

func getNextPlaybookFilename() string {
	const ymlPlaybookAction = "playbook-action"
	ymlPlaybookSeq++
	return ymlPlaybookAction + strconv.Itoa(ymlPlaybookSeq) + ".yml"
}

func (p Provider) StartNodes(ctx context.Context, nodes ...*e2e.Node) error {
	nodeIPs := make([]string, len(nodes))
	for i, n := range nodes {
		nodeIPs[i] = n.ExternalIP.String()
	}
	playbook := ansibleSystemdBytes(true)
	playbookFile := getNextPlaybookFilename()
	if err := p.writePlaybook(playbookFile, playbook); err != nil {
		return err
	}

	return execAnsible(ctx, p.Testnet.Dir, playbookFile, nodeIPs)
}

// Currently unsupported.
func (Provider) SetLatency(_ context.Context, _ *e2e.Node) error {
	return errors.New("SetLatency() currently unsupported for Digital Ocean")
}

func (p Provider) StopTestnet(ctx context.Context) error {
	nodeIPs := make([]string, len(p.Testnet.Nodes))
	for i, n := range p.Testnet.Nodes {
		nodeIPs[i] = n.ExternalIP.String()
	}

	playbook := ansibleSystemdBytes(false)
	playbookFile := getNextPlaybookFilename()
	if err := p.writePlaybook(playbookFile, playbook); err != nil {
		return err
	}
	return execAnsible(ctx, p.Testnet.Dir, playbookFile, nodeIPs)
}

func (p Provider) Disconnect(ctx context.Context, _ string, ip string) error {
	playbook := ansiblePerturbConnectionBytes(true)
	playbookFile := getNextPlaybookFilename()
	if err := p.writePlaybook(playbookFile, playbook); err != nil {
		return err
	}
	return execAnsible(ctx, p.Testnet.Dir, playbookFile, []string{ip})
}

func (p Provider) Reconnect(ctx context.Context, _ string, ip string) error {
	playbook := ansiblePerturbConnectionBytes(false)
	playbookFile := getNextPlaybookFilename()
	if err := p.writePlaybook(playbookFile, playbook); err != nil {
		return err
	}
	return execAnsible(ctx, p.Testnet.Dir, playbookFile, []string{ip})
}

func (Provider) CheckUpgraded(_ context.Context, node *e2e.Node) (string, bool, error) {
	// Upgrade not supported yet by DO provider
	return node.Name, false, nil
}

func (p Provider) writePlaybook(yaml, playbook string) error {
	//nolint: gosec
	// G306: Expect WriteFile permissions to be 0600 or less
	err := os.WriteFile(filepath.Join(p.Testnet.Dir, yaml), []byte(playbook), 0o644)
	if err != nil {
		return err
	}
	return nil
}

const basePlaybook = `- name: e2e custom playbook
  hosts: all
  gather_facts: yes
  vars:
    ansible_host_key_checking: false

  tasks:
`

func ansibleAddTask(playbook, name, contents string) string {
	return playbook + "  - name: " + name + "\n" + contents
}

func ansibleAddSystemdTask(playbook string, starting bool) string {
	startStop := "stopped"
	if starting {
		startStop = "started"
	}
	contents := fmt.Sprintf(`    ansible.builtin.systemd:
      name: testappd
      state: %s
      enabled: yes`, startStop)

	return ansibleAddTask(playbook, "operate on the systemd-unit", contents)
}

func ansibleAddShellTasks(playbook, name string, shells ...string) string {
	for _, shell := range shells {
		contents := fmt.Sprintf("    shell: \"%s\"\n", shell)
		playbook = ansibleAddTask(playbook, name, contents)
	}
	return playbook
}

// file as bytes to be written out to disk.
// ansibleStartBytes generates an Ansible playbook to start the network.
func ansibleSystemdBytes(starting bool) string {
	return ansibleAddSystemdTask(basePlaybook, starting)
}

func ansiblePerturbConnectionBytes(disconnect bool) string {
	disconnecting := "reconnect"
	op := "-D"
	if disconnect {
		disconnecting = "disconnect"
		op = "-A"
	}
	playbook := basePlaybook
	for _, dir := range []string{"INPUT", "OUTPUT"} {
		playbook = ansibleAddShellTasks(playbook, disconnecting+" node",
			fmt.Sprintf("iptables %s %s -p tcp --dport 26656 -j DROP", op, dir))
	}
	return playbook
}

// ExecCompose runs a Docker Compose command for a testnet.
func execAnsible(ctx context.Context, dir, playbook string, nodeIPs []string, args ...string) error { //nolint:unparam
	playbook = filepath.Join(dir, playbook)
	return exec.CommandVerbose(ctx, append(
		[]string{"ansible-playbook", playbook, "-f", "50", "-u", "root", "--inventory", strings.Join(nodeIPs, ",") + ","},
		args...)...)
}
