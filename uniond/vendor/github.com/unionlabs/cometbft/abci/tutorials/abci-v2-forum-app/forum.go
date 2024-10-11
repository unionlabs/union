package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
	"time"

	"github.com/spf13/viper"

	"github.com/cometbft/cometbft/abci/tutorials/abci-v2-forum-app/abci"
	cfg "github.com/cometbft/cometbft/config"
	cmtflags "github.com/cometbft/cometbft/libs/cli/flags"
	cmtlog "github.com/cometbft/cometbft/libs/log"
	nm "github.com/cometbft/cometbft/node"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/privval"
	"github.com/cometbft/cometbft/proxy"
)

var homeDir string

func init() {
	flag.StringVar(&homeDir, "home", "", "Path to the CometBFT config directory (if empty, uses $HOME/.forumapp)")
}

func main() {
	flag.Parse()
	if homeDir == "" {
		homeDir = os.ExpandEnv("$HOME/.forumapp")
	}

	config := cfg.DefaultConfig()
	config.SetRoot(homeDir)
	viper.SetConfigFile(fmt.Sprintf("%s/%s", homeDir, "config/config.toml"))

	if err := viper.ReadInConfig(); err != nil {
		log.Fatalf("failed to read config: %v", err)
	}

	logger := cmtlog.NewTMLogger(cmtlog.NewSyncWriter(os.Stdout))
	logger, err := cmtflags.ParseLogLevel(config.LogLevel, logger, cfg.DefaultLogLevel)
	if err != nil {
		panic(fmt.Errorf("failed to parse log level: %w", err))
	}

	dbPath := filepath.Join(homeDir, "forum-db")
	appConfigPath := "app.toml"
	app, err := abci.NewForumApp(dbPath, appConfigPath, logger)
	if err != nil {
		panic(fmt.Errorf("failed to create Forum Application: %w", err))
	}

	nodeKey, err := p2p.LoadNodeKey(config.NodeKeyFile())
	if err != nil {
		panic(fmt.Errorf("failed to load node key: %w", err))
	}

	pv := privval.LoadFilePV(
		config.PrivValidatorKeyFile(),
		config.PrivValidatorStateFile(),
	)

	node, err := nm.NewNode(
		context.Background(),
		config,
		pv,
		nodeKey,
		proxy.NewLocalClientCreator(app),
		nm.DefaultGenesisDocProviderFunc(config),
		cfg.DefaultDBProvider,
		nm.DefaultMetricsProvider(config.Instrumentation),
		logger,
	)

	defer func() {
		_ = node.Stop()
		node.Wait()
	}()

	if err != nil {
		panic(fmt.Errorf("failed to create CometBFT node: %w", err))
	}

	if err := node.Start(); err != nil {
		panic(fmt.Errorf("failed to start CometBFT node: %w", err))
	}

	httpAddr := "127.0.0.1:8080"

	server := &http.Server{
		Addr:              httpAddr,
		ReadHeaderTimeout: 5 * time.Second,
	}

	if err := server.ListenAndServe(); err != nil {
		panic(fmt.Errorf("failed to start HTTP server: %w", err))
	}

	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
	<-sigCh

	log.Println("Forum application stopped")
}
