module github.com/prysmaticlabs/prysm/v4

go 1.21

toolchain go1.21.5

require (
	contrib.go.opencensus.io/exporter/jaeger v0.2.1
	github.com/MariusVanDerWijden/FuzzyVM v0.0.0-20221202121132-bd37e8fb1d0d
	github.com/MariusVanDerWijden/tx-fuzz v1.0.2
	github.com/aristanetworks/goarista v0.0.0-20200805130819-fd197cf57d96
	github.com/bazelbuild/rules_go v0.23.2
	github.com/btcsuite/btcd/btcec/v2 v2.3.2
	github.com/consensys/gnark-crypto v0.12.1
	github.com/crate-crypto/go-kzg-4844 v0.7.0
	github.com/d4l3k/messagediff v1.2.1
	github.com/dgraph-io/ristretto v0.0.4-0.20210318174700-74754f61e018
	github.com/dustin/go-humanize v1.0.0
	github.com/emicklei/dot v0.11.0
	github.com/ethereum/go-ethereum v1.13.5-0.20231027145059-2d7dba024d76
	github.com/fjl/memsize v0.0.0-20190710130421-bcb5799ab5e5
	github.com/fsnotify/fsnotify v1.6.0
	github.com/ghodss/yaml v1.0.0
	github.com/go-yaml/yaml v2.1.0+incompatible
	github.com/gogo/protobuf v1.3.2
	github.com/golang-jwt/jwt/v4 v4.5.0
	github.com/golang/gddo v0.0.0-20200528160355-8d077c1d8f4c
	github.com/golang/mock v1.6.0
	github.com/golang/protobuf v1.5.3
	github.com/golang/snappy v0.0.5-0.20220116011046-fa5810519dcb
	github.com/google/gofuzz v1.2.0
	github.com/google/uuid v1.3.0
	github.com/gorilla/mux v1.8.0
	github.com/gostaticanalysis/comment v1.4.2
	github.com/grpc-ecosystem/go-grpc-middleware v1.2.2
	github.com/grpc-ecosystem/go-grpc-prometheus v1.2.0
	github.com/grpc-ecosystem/grpc-gateway/v2 v2.0.1
	github.com/hashicorp/golang-lru v0.5.5-0.20210104140557-80c98217689d
	github.com/herumi/bls-eth-go-binary v0.0.0-20210917013441-d37c07cfda4e
	github.com/holiman/uint256 v1.2.3
	github.com/ianlancetaylor/cgosymbolizer v0.0.0-20200424224625-be1b05b0b279
	github.com/ipfs/go-log/v2 v2.5.1
	github.com/joonix/log v0.0.0-20200409080653-9c1d2ceb5f1d
	github.com/json-iterator/go v1.1.12
	github.com/k0kubun/go-ansi v0.0.0-20180517002512-3bf9e2903213
	github.com/kr/pretty v0.3.1
	github.com/libp2p/go-libp2p v0.32.1
	github.com/libp2p/go-libp2p-mplex v0.9.0
	github.com/libp2p/go-libp2p-pubsub v0.10.0
	github.com/logrusorgru/aurora v2.0.3+incompatible
	github.com/manifoldco/promptui v0.7.0
	github.com/mgutz/ansi v0.0.0-20170206155736-9520e82c474b
	github.com/minio/highwayhash v1.0.2
	github.com/minio/sha256-simd v1.0.1
	github.com/mohae/deepcopy v0.0.0-20170929034955-c48cc78d4826
	github.com/multiformats/go-multiaddr v0.12.1
	github.com/onsi/ginkgo v1.16.5
	github.com/onsi/gomega v1.27.10
	github.com/patrickmn/go-cache v2.1.0+incompatible
	github.com/paulbellamy/ratecounter v0.2.0
	github.com/pborman/uuid v1.2.1
	github.com/pkg/errors v0.9.1
	github.com/prometheus/client_golang v1.14.0
	github.com/prometheus/client_model v0.4.0
	github.com/prometheus/prom2json v1.3.0
	github.com/prysmaticlabs/fastssz v0.0.0-20221107182844-78142813af44
	github.com/prysmaticlabs/go-bitfield v0.0.0-20210809151128-385d8c5e3fb7
	github.com/prysmaticlabs/prombbolt v0.0.0-20210126082820-9b7adba6db7c
	github.com/prysmaticlabs/protoc-gen-go-cast v0.0.0-20230228205207-28762a7b9294
	github.com/rs/cors v1.7.0
	github.com/schollz/progressbar/v3 v3.3.4
	github.com/sirupsen/logrus v1.9.0
	github.com/spf13/afero v1.10.0
	github.com/status-im/keycard-go v0.2.0
	github.com/stretchr/testify v1.8.4
	github.com/supranational/blst v0.3.11
	github.com/thomaso-mirodin/intmath v0.0.0-20160323211736-5dc6d854e46e
	github.com/trailofbits/go-mutexasserts v0.0.0-20230328101604-8cdbc5f3d279
	github.com/tyler-smith/go-bip39 v1.1.0
	github.com/urfave/cli/v2 v2.25.7
	github.com/uudashr/gocognit v1.0.5
	github.com/wealdtech/go-bytesutil v1.1.1
	github.com/wealdtech/go-eth2-util v1.6.3
	github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4 v1.1.3
	go.etcd.io/bbolt v1.3.6
	go.opencensus.io v0.24.0
	go.uber.org/automaxprocs v1.5.2
	golang.org/x/crypto v0.17.0
	golang.org/x/exp v0.0.0-20231214170342-aacd6d4b4611
	golang.org/x/mod v0.14.0
	golang.org/x/sync v0.5.0
	golang.org/x/tools v0.16.0
	google.golang.org/genproto v0.0.0-20230410155749-daa745c078e1
	google.golang.org/grpc v1.56.3
	google.golang.org/protobuf v1.30.0
	gopkg.in/d4l3k/messagediff.v1 v1.2.1
	gopkg.in/yaml.v2 v2.4.0
	gopkg.in/yaml.v3 v3.0.1
	honnef.co/go/tools v0.5.0-0.dev.0.20231205170804-aef76f4feee2
	k8s.io/apimachinery v0.20.0
	k8s.io/client-go v0.20.0
)

require (
	github.com/BurntSushi/toml v1.3.2 // indirect
	github.com/DataDog/zstd v1.5.2 // indirect
	github.com/Microsoft/go-winio v0.6.1 // indirect
	github.com/VictoriaMetrics/fastcache v1.12.1 // indirect
	github.com/benbjohnson/clock v1.3.5 // indirect
	github.com/beorn7/perks v1.0.1 // indirect
	github.com/bits-and-blooms/bitset v1.7.0 // indirect
	github.com/cespare/cp v1.1.1 // indirect
	github.com/cespare/xxhash v1.1.0 // indirect
	github.com/cespare/xxhash/v2 v2.2.0 // indirect
	github.com/chzyer/readline v1.5.1 // indirect
	github.com/cockroachdb/errors v1.9.1 // indirect
	github.com/cockroachdb/logtags v0.0.0-20230118201751-21c54148d20b // indirect
	github.com/cockroachdb/pebble v0.0.0-20230928194634-aa077af62593 // indirect
	github.com/cockroachdb/redact v1.1.3 // indirect
	github.com/cockroachdb/tokenbucket v0.0.0-20230807174530-cc333fc44b06 // indirect
	github.com/consensys/bavard v0.1.13 // indirect
	github.com/containerd/cgroups v1.1.0 // indirect
	github.com/coreos/go-systemd/v22 v22.5.0 // indirect
	github.com/cpuguy83/go-md2man/v2 v2.0.2 // indirect
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/davidlazar/go-crypto v0.0.0-20200604182044-b73af7476f6c // indirect
	github.com/deckarep/golang-set/v2 v2.1.0 // indirect
	github.com/decred/dcrd/dcrec/secp256k1/v4 v4.2.0 // indirect
	github.com/deepmap/oapi-codegen v1.8.2 // indirect
	github.com/dlclark/regexp2 v1.7.0 // indirect
	github.com/docker/go-units v0.5.0 // indirect
	github.com/dop251/goja v0.0.0-20230806174421-c933cf95e127 // indirect
	github.com/elastic/gosigar v0.14.2 // indirect
	github.com/ethereum/c-kzg-4844 v0.4.0 // indirect
	github.com/ferranbt/fastssz v0.0.0-20210120143747-11b9eff30ea9 // indirect
	github.com/flynn/noise v1.0.0 // indirect
	github.com/francoispqt/gojay v1.2.13 // indirect
	github.com/getsentry/sentry-go v0.18.0 // indirect
	github.com/go-playground/locales v0.14.1 // indirect
	github.com/go-playground/universal-translator v0.18.1 // indirect
	github.com/go-sourcemap/sourcemap v2.1.3+incompatible // indirect
	github.com/go-stack/stack v1.8.1 // indirect
	github.com/go-task/slim-sprig v0.0.0-20230315185526-52ccab3ef572 // indirect
	github.com/godbus/dbus/v5 v5.1.0 // indirect
	github.com/gofrs/flock v0.8.1 // indirect
	github.com/golang/groupcache v0.0.0-20200121045136-8c9f03a8e57e // indirect
	github.com/google/go-cmp v0.5.9 // indirect
	github.com/google/gopacket v1.1.19 // indirect
	github.com/google/pprof v0.0.0-20231023181126-ff6d637d2a7b // indirect
	github.com/gorilla/websocket v1.5.0 // indirect
	github.com/graph-gophers/graphql-go v1.3.0 // indirect
	github.com/hashicorp/go-bexpr v0.1.10 // indirect
	github.com/hashicorp/golang-lru/v2 v2.0.5 // indirect
	github.com/holiman/billy v0.0.0-20230718173358-1c7e68d277a7 // indirect
	github.com/holiman/bloomfilter/v2 v2.0.3 // indirect
	github.com/holiman/goevmlab v0.0.0-20230917164918-f3777d0b880b // indirect
	github.com/huin/goupnp v1.3.0 // indirect
	github.com/influxdata/influxdb-client-go/v2 v2.4.0 // indirect
	github.com/influxdata/influxdb1-client v0.0.0-20220302092344-a9ab5670611c // indirect
	github.com/influxdata/line-protocol v0.0.0-20210311194329-9aa0e372d097 // indirect
	github.com/ipfs/go-cid v0.4.1 // indirect
	github.com/jackpal/go-nat-pmp v1.0.2 // indirect
	github.com/jbenet/go-temp-err-catcher v0.1.0 // indirect
	github.com/juju/ansiterm v0.0.0-20180109212912-720a0952cc2a // indirect
	github.com/karalabe/usb v0.0.3-0.20230711191512-61db3e06439c // indirect
	github.com/klauspost/compress v1.17.2 // indirect
	github.com/klauspost/cpuid/v2 v2.2.5 // indirect
	github.com/koron/go-ssdp v0.0.4 // indirect
	github.com/kr/text v0.2.0 // indirect
	github.com/leodido/go-urn v1.2.3 // indirect
	github.com/libp2p/go-buffer-pool v0.1.0 // indirect
	github.com/libp2p/go-flow-metrics v0.1.0 // indirect
	github.com/libp2p/go-libp2p-asn-util v0.4.1 // indirect
	github.com/libp2p/go-mplex v0.7.0 // indirect
	github.com/libp2p/go-msgio v0.3.0 // indirect
	github.com/libp2p/go-nat v0.2.0 // indirect
	github.com/libp2p/go-netroute v0.2.1 // indirect
	github.com/libp2p/go-reuseport v0.4.0 // indirect
	github.com/libp2p/go-yamux/v4 v4.0.1 // indirect
	github.com/lunixbochs/vtclean v1.0.0 // indirect
	github.com/marten-seemann/tcp v0.0.0-20210406111302-dfbc87cc63fd // indirect
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-isatty v0.0.20 // indirect
	github.com/mattn/go-runewidth v0.0.14 // indirect
	github.com/matttproud/golang_protobuf_extensions v1.0.4 // indirect
	github.com/miekg/dns v1.1.56 // indirect
	github.com/mikioh/tcpinfo v0.0.0-20190314235526-30a79bb1804b // indirect
	github.com/mikioh/tcpopt v0.0.0-20190314235656-172688c1accc // indirect
	github.com/mitchellh/colorstring v0.0.0-20190213212951-d06e56a500db // indirect
	github.com/mitchellh/mapstructure v1.4.1 // indirect
	github.com/mitchellh/pointerstructure v1.2.0 // indirect
	github.com/mmcloughlin/addchain v0.4.0 // indirect
	github.com/modern-go/concurrent v0.0.0-20180306012644-bacd9c7ef1dd // indirect
	github.com/modern-go/reflect2 v1.0.2 // indirect
	github.com/mr-tron/base58 v1.2.0 // indirect
	github.com/multiformats/go-base32 v0.1.0 // indirect
	github.com/multiformats/go-base36 v0.2.0 // indirect
	github.com/multiformats/go-multiaddr-dns v0.3.1 // indirect
	github.com/multiformats/go-multiaddr-fmt v0.1.0 // indirect
	github.com/multiformats/go-multibase v0.2.0 // indirect
	github.com/multiformats/go-multicodec v0.9.0 // indirect
	github.com/multiformats/go-multihash v0.2.3 // indirect
	github.com/multiformats/go-multistream v0.5.0 // indirect
	github.com/multiformats/go-varint v0.0.7 // indirect
	github.com/nxadm/tail v1.4.8 // indirect
	github.com/olekukonko/tablewriter v0.0.5 // indirect
	github.com/onsi/ginkgo/v2 v2.13.0 // indirect
	github.com/opencontainers/runtime-spec v1.1.0 // indirect
	github.com/opentracing/opentracing-go v1.2.0 // indirect
	github.com/pbnjay/memory v0.0.0-20210728143218-7b4eea64cf58 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/prometheus/common v0.42.0 // indirect
	github.com/prometheus/procfs v0.9.0 // indirect
	github.com/quic-go/qpack v0.4.0 // indirect
	github.com/quic-go/qtls-go1-20 v0.3.4 // indirect
	github.com/quic-go/quic-go v0.39.4 // indirect
	github.com/quic-go/webtransport-go v0.6.0 // indirect
	github.com/raulk/go-watchdog v1.3.0 // indirect
	github.com/rivo/uniseg v0.4.4 // indirect
	github.com/rogpeppe/go-internal v1.9.0 // indirect
	github.com/russross/blackfriday/v2 v2.1.0 // indirect
	github.com/shirou/gopsutil v3.21.11+incompatible // indirect
	github.com/spaolacci/murmur3 v1.1.0 // indirect
	github.com/stretchr/objx v0.5.0 // indirect
	github.com/syndtr/goleveldb v1.0.1-0.20220721030215-126854af5e6d // indirect
	github.com/tklauser/go-sysconf v0.3.12 // indirect
	github.com/tklauser/numcpus v0.6.1 // indirect
	github.com/uber/jaeger-client-go v2.25.0+incompatible // indirect
	github.com/wealdtech/go-eth2-types/v2 v2.5.2 // indirect
	github.com/xrash/smetrics v0.0.0-20201216005158-039620a65673 // indirect
	github.com/yusufpapurcu/wmi v1.2.2 // indirect
	go.uber.org/dig v1.17.1 // indirect
	go.uber.org/fx v1.20.1 // indirect
	go.uber.org/mock v0.3.0 // indirect
	go.uber.org/multierr v1.11.0 // indirect
	go.uber.org/zap v1.26.0 // indirect
	golang.org/x/exp/typeparams v0.0.0-20231108232855-2478ac86f678 // indirect
	golang.org/x/net v0.19.0 // indirect
	golang.org/x/oauth2 v0.7.0 // indirect
	golang.org/x/term v0.15.0 // indirect
	golang.org/x/text v0.14.0 // indirect
	golang.org/x/time v0.3.0 // indirect
	gopkg.in/inf.v0 v0.9.1 // indirect
	gopkg.in/natefinch/lumberjack.v2 v2.0.0 // indirect
	gopkg.in/tomb.v1 v1.0.0-20141024135613-dd632973f1e7 // indirect
	lukechampine.com/blake3 v1.2.1 // indirect
	rsc.io/tmplfunc v0.0.3 // indirect
	sigs.k8s.io/structured-merge-diff/v4 v4.0.2 // indirect
	sigs.k8s.io/yaml v1.2.0 // indirect
)

require (
	github.com/coreos/go-systemd v0.0.0-20191104093116-d3cd4ed1dbcf
	github.com/fatih/color v1.13.0 // indirect
	github.com/gballet/go-libpcsclite v0.0.0-20191108122812-4678299bea08 // indirect
	github.com/go-logr/logr v1.2.4 // indirect
	github.com/go-ole/go-ole v1.2.6 // indirect
	github.com/go-playground/validator/v10 v10.13.0
	github.com/peterh/liner v1.2.0 // indirect
	github.com/prysmaticlabs/gohashtree v0.0.3-alpha
	golang.org/x/sys v0.15.0 // indirect
	google.golang.org/api v0.44.0 // indirect
	google.golang.org/appengine v1.6.7 // indirect
	k8s.io/klog/v2 v2.80.0 // indirect
	k8s.io/utils v0.0.0-20201110183641-67b214c5f920 // indirect
)

replace github.com/json-iterator/go => github.com/prestonvanloon/go v1.1.7-0.20190722034630-4f2e55fcf87b

// See https://github.com/prysmaticlabs/grpc-gateway/issues/2
replace github.com/grpc-ecosystem/grpc-gateway/v2 => github.com/prysmaticlabs/grpc-gateway/v2 v2.3.1-0.20230315201114-09284ba20446
