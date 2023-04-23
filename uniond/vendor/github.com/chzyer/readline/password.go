package readline

type opPassword struct {
	o         *Operation
	backupCfg *Config
}

func newOpPassword(o *Operation) *opPassword {
	return &opPassword{o: o}
}

func (o *opPassword) ExitPasswordMode() {
	o.o.SetConfig(o.backupCfg)
	o.backupCfg = nil
}

func (o *opPassword) EnterPasswordMode(cfg *Config) (err error) {
	o.backupCfg, err = o.o.SetConfig(cfg)
	return
}

func (o *opPassword) PasswordConfig() *Config {
	return &Config{
		EnableMask:      true,
		InterruptPrompt: "\n",
		EOFPrompt:       "\n",
		HistoryLimit:    -1,
		Painter:         &defaultPainter{},

		Stdout: o.o.cfg.Stdout,
		Stderr: o.o.cfg.Stderr,
	}
}
