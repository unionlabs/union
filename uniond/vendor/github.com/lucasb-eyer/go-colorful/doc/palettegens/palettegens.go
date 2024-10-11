package main

import "fmt"
import "github.com/lucasb-eyer/go-colorful"
import "image"
import "image/draw"
import "image/png"
import "math/rand"
import "os"
import "time"

func main() {
	colors := 10
	blockw := 40
	space := 5

	rand.Seed(time.Now().UTC().UnixNano())
	img := image.NewRGBA(image.Rect(0, 0, colors*blockw+space*(colors-1), 6*blockw+8*space))

	warm, err := colorful.WarmPalette(colors)
	if err != nil {
		fmt.Printf("Error generating warm palette: %v", err)
		return
	}
	fwarm := colorful.FastWarmPalette(colors)
	happy, err := colorful.HappyPalette(colors)
	if err != nil {
		fmt.Printf("Error generating happy palette: %v", err)
		return
	}
	fhappy := colorful.FastHappyPalette(colors)
	soft, err := colorful.SoftPalette(colors)
	if err != nil {
		fmt.Printf("Error generating soft palette: %v", err)
		return
	}
	brownies, err := colorful.SoftPaletteEx(colors, colorful.SoftPaletteSettings{isbrowny, 50, true})
	if err != nil {
		fmt.Printf("Error generating brownies: %v", err)
		return
	}
	for i := 0; i < colors; i++ {
		draw.Draw(img, image.Rect(i*(blockw+space), 0, i*(blockw+space)+blockw, blockw), &image.Uniform{warm[i]}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*(blockw+space), 1*blockw+1*space, i*(blockw+space)+blockw, 2*blockw+1*space), &image.Uniform{fwarm[i]}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*(blockw+space), 2*blockw+3*space, i*(blockw+space)+blockw, 3*blockw+3*space), &image.Uniform{happy[i]}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*(blockw+space), 3*blockw+4*space, i*(blockw+space)+blockw, 4*blockw+4*space), &image.Uniform{fhappy[i]}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*(blockw+space), 4*blockw+6*space, i*(blockw+space)+blockw, 5*blockw+6*space), &image.Uniform{soft[i]}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*(blockw+space), 5*blockw+8*space, i*(blockw+space)+blockw, 6*blockw+8*space), &image.Uniform{brownies[i]}, image.Point{}, draw.Src)
	}

	toimg, err := os.Create("palettegens.png")
	if err != nil {
		fmt.Printf("Error: %v", err)
		return
	}
	defer toimg.Close()

	png.Encode(toimg, img)
}

func isbrowny(l, a, b float64) bool {
	h, c, L := colorful.LabToHcl(l, a, b)
	return 10.0 < h && h < 50.0 && 0.1 < c && c < 0.5 && L < 0.5
}
