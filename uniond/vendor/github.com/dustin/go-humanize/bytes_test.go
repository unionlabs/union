package humanize

import (
	"testing"
)

func TestByteParsing(t *testing.T) {
	tests := []struct {
		in  string
		exp uint64
	}{
		{"42", 42},
		{"42MB", 42000000},
		{"42MiB", 44040192},
		{"42mb", 42000000},
		{"42mib", 44040192},
		{"42MIB", 44040192},
		{"42 MB", 42000000},
		{"42 MiB", 44040192},
		{"42 mb", 42000000},
		{"42 mib", 44040192},
		{"42 MIB", 44040192},
		{"42.5MB", 42500000},
		{"42.5MiB", 44564480},
		{"42.5 MB", 42500000},
		{"42.5 MiB", 44564480},
		// No need to say B
		{"42M", 42000000},
		{"42Mi", 44040192},
		{"42m", 42000000},
		{"42mi", 44040192},
		{"42MI", 44040192},
		{"42 M", 42000000},
		{"42 Mi", 44040192},
		{"42 m", 42000000},
		{"42 mi", 44040192},
		{"42 MI", 44040192},
		{"42.5M", 42500000},
		{"42.5Mi", 44564480},
		{"42.5 M", 42500000},
		{"42.5 Mi", 44564480},
		// Bug #42
		{"1,005.03 MB", 1005030000},
		// Large testing, breaks when too much larger than
		// this.
		{"12.5 EB", uint64(12.5 * float64(EByte))},
		{"12.5 E", uint64(12.5 * float64(EByte))},
		{"12.5 EiB", uint64(12.5 * float64(EiByte))},
	}

	for _, p := range tests {
		got, err := ParseBytes(p.in)
		if err != nil {
			t.Errorf("Couldn't parse %v: %v", p.in, err)
		}
		if got != p.exp {
			t.Errorf("Expected %v for %v, got %v",
				p.exp, p.in, got)
		}
	}
}

func TestByteErrors(t *testing.T) {
	got, err := ParseBytes("84 JB")
	if err == nil {
		t.Errorf("Expected error, got %v", got)
	}
	_, err = ParseBytes("")
	if err == nil {
		t.Errorf("Expected error parsing nothing")
	}
	got, err = ParseBytes("16 EiB")
	if err == nil {
		t.Errorf("Expected error, got %v", got)
	}
}

func TestBytes(t *testing.T) {
	testList{
		{"bytes(0)", Bytes(0), "0 B"},
		{"bytes(1)", Bytes(1), "1 B"},
		{"bytes(803)", Bytes(803), "803 B"},
		{"bytes(999)", Bytes(999), "999 B"},

		{"bytes(1024)", Bytes(1024), "1.0 kB"},
		{"bytes(9999)", Bytes(9999), "10 kB"},
		{"bytes(1MB - 1)", Bytes(MByte - Byte), "1000 kB"},

		{"bytes(1MB)", Bytes(1024 * 1024), "1.0 MB"},
		{"bytes(1GB - 1K)", Bytes(GByte - KByte), "1000 MB"},

		{"bytes(1GB)", Bytes(GByte), "1.0 GB"},
		{"bytes(1TB - 1M)", Bytes(TByte - MByte), "1000 GB"},
		{"bytes(10MB)", Bytes(9999 * 1000), "10 MB"},

		{"bytes(1TB)", Bytes(TByte), "1.0 TB"},
		{"bytes(1PB - 1T)", Bytes(PByte - TByte), "999 TB"},

		{"bytes(1PB)", Bytes(PByte), "1.0 PB"},
		{"bytes(1PB - 1T)", Bytes(EByte - PByte), "999 PB"},

		{"bytes(1EB)", Bytes(EByte), "1.0 EB"},
		// Overflows.
		// {"bytes(1EB - 1P)", Bytes((KByte*EByte)-PByte), "1023EB"},

		{"bytes(0)", IBytes(0), "0 B"},
		{"bytes(1)", IBytes(1), "1 B"},
		{"bytes(803)", IBytes(803), "803 B"},
		{"bytes(1023)", IBytes(1023), "1023 B"},

		{"bytes(1024)", IBytes(1024), "1.0 KiB"},
		{"bytes(1MB - 1)", IBytes(MiByte - IByte), "1024 KiB"},

		{"bytes(1MB)", IBytes(1024 * 1024), "1.0 MiB"},
		{"bytes(1GB - 1K)", IBytes(GiByte - KiByte), "1024 MiB"},

		{"bytes(1GB)", IBytes(GiByte), "1.0 GiB"},
		{"bytes(1TB - 1M)", IBytes(TiByte - MiByte), "1024 GiB"},

		{"bytes(1TB)", IBytes(TiByte), "1.0 TiB"},
		{"bytes(1PB - 1T)", IBytes(PiByte - TiByte), "1023 TiB"},

		{"bytes(1PB)", IBytes(PiByte), "1.0 PiB"},
		{"bytes(1PB - 1T)", IBytes(EiByte - PiByte), "1023 PiB"},

		{"bytes(1EiB)", IBytes(EiByte), "1.0 EiB"},
		// Overflows.
		// {"bytes(1EB - 1P)", IBytes((KIByte*EIByte)-PiByte), "1023EB"},

		{"bytes(5.5GiB)", IBytes(5.5 * GiByte), "5.5 GiB"},

		{"bytes(5.5GB)", Bytes(5.5 * GByte), "5.5 GB"},
	}.validate(t)
}

func BenchmarkParseBytes(b *testing.B) {
	for i := 0; i < b.N; i++ {
		ParseBytes("16.5 GB")
	}
}

func BenchmarkBytes(b *testing.B) {
	for i := 0; i < b.N; i++ {
		Bytes(16.5 * GByte)
	}
}
