package zstd

import (
	"bytes"
	"io"
	"os"
	"testing"

	snappy "github.com/klauspost/compress/internal/snapref"
)

func TestSnappy_ConvertSimple(t *testing.T) {
	in, err := os.ReadFile("testdata/z000028")
	if err != nil {
		t.Fatal(err)
	}

	var comp bytes.Buffer
	w := snappy.NewBufferedWriter(&comp)
	_, err = io.Copy(w, bytes.NewBuffer(in))
	if err != nil {
		t.Fatal(err)
	}
	err = w.Close()
	if err != nil {
		t.Fatal(err)
	}
	snapLen := comp.Len()
	s := SnappyConverter{}
	var dst bytes.Buffer
	n, err := s.Convert(&comp, &dst)
	if err != io.EOF {
		t.Fatal(err)
	}
	if n != int64(dst.Len()) {
		t.Errorf("Dest was %d bytes, but said to have written %d bytes", dst.Len(), n)
	}
	t.Log("SnappyConverter len", snapLen, "-> zstd len", dst.Len())

	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	decoded, err := dec.DecodeAll(dst.Bytes(), nil)
	if err != nil {
		t.Error(err, len(decoded))
	}
	if !bytes.Equal(decoded, in) {
		os.WriteFile("testdata/z000028.got", decoded, os.ModePerm)
		t.Fatal("Decoded does not match")
	}
	t.Log("Encoded content matched")
}

func TestSnappy_ConvertXML(t *testing.T) {
	f, err := os.Open("testdata/xml.zst")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()

	dec, err := NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	in, err := io.ReadAll(dec)
	if err != nil {
		t.Fatal(err)
	}
	var comp bytes.Buffer
	w := snappy.NewBufferedWriter(&comp)
	_, err = io.Copy(w, bytes.NewBuffer(in))
	if err != nil {
		t.Fatal(err)
	}
	err = w.Close()
	if err != nil {
		t.Fatal(err)
	}
	snapLen := comp.Len()
	s := SnappyConverter{}
	var dst bytes.Buffer
	n, err := s.Convert(&comp, &dst)
	if err != io.EOF {
		t.Fatal(err)
	}
	if n != int64(dst.Len()) {
		t.Errorf("Dest was %d bytes, but said to have written %d bytes", dst.Len(), n)
	}
	t.Log("Snappy len", snapLen, "-> zstd len", dst.Len())

	decoded, err := dec.DecodeAll(dst.Bytes(), nil)
	if err != nil {
		t.Error(err, len(decoded))
	}
	if !bytes.Equal(decoded, in) {
		os.WriteFile("testdata/xml.got", decoded, os.ModePerm)
		t.Fatal("Decoded does not match")
	}
	t.Log("Encoded content matched")
}

func TestSnappy_ConvertSilesia(t *testing.T) {
	if testing.Short() {
		t.SkipNow()
	}
	in, err := os.ReadFile("testdata/silesia.tar")
	if err != nil {
		if os.IsNotExist(err) {
			t.Skip("Missing testdata/silesia.tar")
			return
		}
		t.Fatal(err)
	}

	var comp bytes.Buffer
	w := snappy.NewBufferedWriter(&comp)
	_, err = io.Copy(w, bytes.NewBuffer(in))
	if err != nil {
		t.Fatal(err)
	}
	err = w.Close()
	if err != nil {
		t.Fatal(err)
	}
	snapLen := comp.Len()
	s := SnappyConverter{}
	var dst bytes.Buffer
	n, err := s.Convert(&comp, &dst)
	if err != io.EOF {
		t.Fatal(err)
	}
	if n != int64(dst.Len()) {
		t.Errorf("Dest was %d bytes, but said to have written %d bytes", dst.Len(), n)
	}
	t.Log("SnappyConverter len", snapLen, "-> zstd len", dst.Len())

	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	decoded, err := dec.DecodeAll(dst.Bytes(), nil)
	if err != nil {
		t.Error(err, len(decoded))
	}
	if !bytes.Equal(decoded, in) {
		os.WriteFile("testdata/silesia.tar.got", decoded, os.ModePerm)
		t.Fatal("Decoded does not match")
	}
	t.Log("Encoded content matched")
}

func TestSnappy_ConvertEnwik9(t *testing.T) {
	if testing.Short() {
		t.SkipNow()
	}
	file := "testdata/enwik9.zst"
	f, err := os.Open(file)
	if err != nil {
		if os.IsNotExist(err) {
			t.Skip("To run extended tests, download http://mattmahoney.net/dc/enwik9.zip unzip it \n" +
				"compress it with 'zstd -15 -T0 enwik9' and place it in " + file)
		}
	}
	defer f.Close()

	dec, err := NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	in, err := io.ReadAll(dec)
	if err != nil {
		t.Fatal(err)
	}

	var comp bytes.Buffer
	w := snappy.NewBufferedWriter(&comp)
	_, err = io.Copy(w, bytes.NewBuffer(in))
	if err != nil {
		t.Fatal(err)
	}
	err = w.Close()
	if err != nil {
		t.Fatal(err)
	}
	snapLen := comp.Len()
	s := SnappyConverter{}
	var dst bytes.Buffer
	n, err := s.Convert(&comp, &dst)
	if err != io.EOF {
		t.Fatal(err)
	}
	if n != int64(dst.Len()) {
		t.Errorf("Dest was %d bytes, but said to have written %d bytes", dst.Len(), n)
	}
	t.Log("SnappyConverter len", snapLen, "-> zstd len", dst.Len())

	decoded, err := dec.DecodeAll(dst.Bytes(), nil)
	if err != nil {
		t.Error(err, len(decoded))
	}
	if !bytes.Equal(decoded, in) {
		os.WriteFile("testdata/enwik9.got", decoded, os.ModePerm)
		t.Fatal("Decoded does not match")
	}
	t.Log("Encoded content matched")
}

func BenchmarkSnappy_ConvertXML(b *testing.B) {
	f, err := os.Open("testdata/xml.zst")
	if err != nil {
		b.Fatal(err)
	}
	defer f.Close()

	dec, err := NewReader(f)
	if err != nil {
		b.Fatal(err)
	}
	defer dec.Close()
	in, err := io.ReadAll(dec)
	if err != nil {
		b.Fatal(err)
	}

	var comp bytes.Buffer
	w := snappy.NewBufferedWriter(&comp)
	_, err = io.Copy(w, bytes.NewBuffer(in))
	if err != nil {
		b.Fatal(err)
	}
	err = w.Close()
	if err != nil {
		b.Fatal(err)
	}
	s := SnappyConverter{}
	compBytes := comp.Bytes()
	_, err = s.Convert(&comp, io.Discard)
	if err != io.EOF {
		b.Fatal(err)
	}
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(in)))
	for i := 0; i < b.N; i++ {
		_, err := s.Convert(bytes.NewBuffer(compBytes), io.Discard)
		if err != io.EOF {
			b.Fatal(err)
		}
	}
}

func BenchmarkSnappy_Enwik9(b *testing.B) {
	f, err := os.Open("testdata/enwik9.zst")
	if err != nil {
		b.Skip(err)
		return
	}
	defer f.Close()

	dec, err := NewReader(f)
	if err != nil {
		b.Fatal(err)
	}
	defer dec.Close()
	in, err := io.ReadAll(dec)
	if err != nil {
		b.Fatal(err)
	}
	defer dec.Close()

	var comp bytes.Buffer
	w := snappy.NewBufferedWriter(&comp)
	_, err = io.Copy(w, bytes.NewBuffer(in))
	if err != nil {
		b.Fatal(err)
	}
	err = w.Close()
	if err != nil {
		b.Fatal(err)
	}
	s := SnappyConverter{}
	compBytes := comp.Bytes()
	_, err = s.Convert(&comp, io.Discard)
	if err != io.EOF {
		b.Fatal(err)
	}
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(in)))
	for i := 0; i < b.N; i++ {
		_, err := s.Convert(bytes.NewBuffer(compBytes), io.Discard)
		if err != io.EOF {
			b.Fatal(err)
		}
	}
}

func BenchmarkSnappy_ConvertSilesia(b *testing.B) {
	in, err := os.ReadFile("testdata/silesia.tar")
	if err != nil {
		if os.IsNotExist(err) {
			b.Skip("Missing testdata/silesia.tar")
			return
		}
		b.Fatal(err)
	}

	var comp bytes.Buffer
	w := snappy.NewBufferedWriter(&comp)
	_, err = io.Copy(w, bytes.NewBuffer(in))
	if err != nil {
		b.Fatal(err)
	}
	err = w.Close()
	if err != nil {
		b.Fatal(err)
	}
	s := SnappyConverter{}
	compBytes := comp.Bytes()
	_, err = s.Convert(&comp, io.Discard)
	if err != io.EOF {
		b.Fatal(err)
	}
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(in)))
	for i := 0; i < b.N; i++ {
		_, err := s.Convert(bytes.NewBuffer(compBytes), io.Discard)
		if err != io.EOF {
			b.Fatal(err)
		}
	}
}
