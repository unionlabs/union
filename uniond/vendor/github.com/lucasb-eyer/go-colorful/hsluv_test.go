package colorful

// Source: https://github.com/hsluv/hsluv-go
// Under MIT License
// Modified so that Saturation and Luminance are in [0..1] instead of [0..100],
// and so that it works with this library in general.

import (
	"encoding/json"
	"fmt"
	"math"
	"os"
	"testing"
)

type mapping map[string]values

type values struct {
	Rgb   [3]float64
	Xyz   [3]float64
	Luv   [3]float64
	Lch   [3]float64
	Hsluv [3]float64
	Hpluv [3]float64
}

func pack(a, b, c float64) [3]float64 {
	return [3]float64{a, b, c}
}

func unpack(tuple [3]float64) (float64, float64, float64) {
	return tuple[0], tuple[1], tuple[2]
}

func fromHex(s string) Color {
	c, _ := Hex(s)
	return c
}

// const delta = 0.00000001
const hsluvTestDelta = 0.0000000001 // Two more zeros than the original delta, because values are divided by 100

func compareTuple(t *testing.T, result, expected [3]float64, method string, hex string) {
	var err bool
	var errs [3]bool
	for i := 0; i < 3; i++ {
		if math.Abs(result[i]-expected[i]) > hsluvTestDelta {
			err = true
			errs[i] = true
		}
	}
	if err {
		resultOutput := "["
		for i := 0; i < 3; i++ {
			resultOutput += fmt.Sprintf("%f", result[i])
			if errs[i] {
				resultOutput += " *"
			}
			if i < 2 {
				resultOutput += ", "
			}
		}
		resultOutput += "]"
		t.Errorf("result: %s expected: %v, testing %s with test case %s", resultOutput, expected, method, hex)
	}
}

func compareHex(t *testing.T, result, expected string, method string, hex string) {
	if result != expected {
		t.Errorf("result: %v expected: %v, testing %s with test case %s", result, expected, method, hex)
	}
}

func TestHSLuv(t *testing.T) {
	snapshotFile, err := os.Open("hsluv-snapshot-rev4.json")
	if err != nil {
		t.Fatal(err)
	}
	defer snapshotFile.Close()

	jsonParser := json.NewDecoder(snapshotFile)
	snapshot := make(mapping)
	if err = jsonParser.Decode(&snapshot); err != nil {
		t.Fatal(err)
	}

	for hex, colorValues := range snapshot {
		// tests for public methods
		if testing.Verbose() {
			t.Logf("Testing public methods for test case %s", hex)
		}

		// Adjust color values to be in the ranges this library uses
		colorValues.Hsluv[1] /= 100.0
		colorValues.Hsluv[2] /= 100.0
		colorValues.Hpluv[1] /= 100.0
		colorValues.Hpluv[2] /= 100.0

		compareHex(t, HSLuv(unpack(colorValues.Hsluv)).Hex(), hex, "HsluvToHex", hex)
		compareTuple(t, pack(HSLuv(unpack(colorValues.Hsluv)).values()), colorValues.Rgb, "HsluvToRGB", hex)
		compareTuple(t, pack(fromHex(hex).HSLuv()), colorValues.Hsluv, "HsluvFromHex", hex)
		compareTuple(t, pack(Color{colorValues.Rgb[0], colorValues.Rgb[1], colorValues.Rgb[2]}.HSLuv()), colorValues.Hsluv, "HsluvFromRGB", hex)
		compareHex(t, HPLuv(unpack(colorValues.Hpluv)).Hex(), hex, "HpluvToHex", hex)
		compareTuple(t, pack(HPLuv(unpack(colorValues.Hpluv)).values()), colorValues.Rgb, "HpluvToRGB", hex)
		compareTuple(t, pack(fromHex(hex).HPLuv()), colorValues.Hpluv, "HpluvFromHex", hex)
		compareTuple(t, pack(Color{colorValues.Rgb[0], colorValues.Rgb[1], colorValues.Rgb[2]}.HPLuv()), colorValues.Hpluv, "HpluvFromRGB", hex)

		if !testing.Short() {
			// internal tests
			if testing.Verbose() {
				t.Logf("Testing internal methods for test case %s", hex)
			}

			// Adjust color values to be in the ranges this library uses
			colorValues.Lch[0] /= 100.0
			colorValues.Lch[1] /= 100.0
			colorValues.Luv[0] /= 100.0
			colorValues.Luv[1] /= 100.0
			colorValues.Luv[2] /= 100.0

			compareTuple(t, pack(LuvLChWhiteRef(
				colorValues.Lch[0], colorValues.Lch[1], colorValues.Lch[2], hSLuvD65,
			).values()), colorValues.Rgb, "convLchRgb", hex)
			compareTuple(t, pack(Color{
				colorValues.Rgb[0], colorValues.Rgb[1], colorValues.Rgb[2],
			}.LuvLChWhiteRef(hSLuvD65)), colorValues.Lch, "convRgbLch", hex)
			compareTuple(t, pack(XyzToLuvWhiteRef(
				colorValues.Xyz[0], colorValues.Xyz[1], colorValues.Xyz[2], hSLuvD65,
			)), colorValues.Luv, "convXyzLuv", hex)
			compareTuple(t, pack(LuvToXyzWhiteRef(
				colorValues.Luv[0], colorValues.Luv[1], colorValues.Luv[2], hSLuvD65,
			)), colorValues.Xyz, "convLuvXyz", hex)
			compareTuple(t, pack(LuvToLuvLCh(unpack(colorValues.Luv))), colorValues.Lch, "convLuvLch", hex)
			compareTuple(t, pack(LuvLChToLuv(unpack(colorValues.Lch))), colorValues.Luv, "convLchLuv", hex)
			compareTuple(t, pack(HSLuvToLuvLCh(unpack(colorValues.Hsluv))), colorValues.Lch, "convHsluvLch", hex)
			compareTuple(t, pack(LuvLChToHSLuv(unpack(colorValues.Lch))), colorValues.Hsluv, "convLchHsluv", hex)
			compareTuple(t, pack(HPLuvToLuvLCh(unpack(colorValues.Hpluv))), colorValues.Lch, "convHpluvLch", hex)
			compareTuple(t, pack(LuvLChToHPLuv(unpack(colorValues.Lch))), colorValues.Hpluv, "convLchHpluv", hex)
			compareTuple(t, pack(LinearRgb(XyzToLinearRgb(unpack(colorValues.Xyz))).values()), colorValues.Rgb, "convXyzRgb", hex)
			compareTuple(t, pack(Color{colorValues.Rgb[0], colorValues.Rgb[1], colorValues.Rgb[2]}.Xyz()), colorValues.Xyz, "convRgbXyz", hex)
		}
	}
}
