package main

import "fmt"
import "github.com/lucasb-eyer/go-colorful"
import "image"
import "image/draw"
import "image/png"
import "os"

func main() {
	blocks := 10
	blockw := 40
	img := image.NewRGBA(image.Rect(0, 0, blocks*blockw, 200))

	c1, _ := colorful.Hex("#fdffcc")
	c2, _ := colorful.Hex("#242a42")

	// Use these colors to get invalid RGB in the gradient.
	//c1, _ := colorful.Hex("#EEEF61")
	//c2, _ := colorful.Hex("#1E3140")

	for i := 0; i < blocks; i++ {
		draw.Draw(img, image.Rect(i*blockw, 0, (i+1)*blockw, 40), &image.Uniform{c1.BlendHsv(c2, float64(i)/float64(blocks-1))}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*blockw, 40, (i+1)*blockw, 80), &image.Uniform{c1.BlendLuv(c2, float64(i)/float64(blocks-1))}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*blockw, 80, (i+1)*blockw, 120), &image.Uniform{c1.BlendRgb(c2, float64(i)/float64(blocks-1))}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*blockw, 120, (i+1)*blockw, 160), &image.Uniform{c1.BlendLab(c2, float64(i)/float64(blocks-1))}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*blockw, 160, (i+1)*blockw, 200), &image.Uniform{c1.BlendHcl(c2, float64(i)/float64(blocks-1))}, image.Point{}, draw.Src)

		// This can be used to "fix" invalid colors in the gradient.
		//draw.Draw(img, image.Rect(i*blockw,160,(i+1)*blockw,200), &image.Uniform{c1.BlendHcl(c2, float64(i)/float64(blocks-1)).Clamped()}, image.Point{}, draw.Src)
	}

	toimg, err := os.Create("colorblend.png")
	if err != nil {
		fmt.Printf("Error: %v", err)
		return
	}
	defer toimg.Close()

	png.Encode(toimg, img)
}
