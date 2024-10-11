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
	blocks := 10
	blockw := 40
	space := 5

	rand.Seed(time.Now().UTC().UnixNano())
	img := image.NewRGBA(image.Rect(0, 0, blocks*blockw+space*(blocks-1), 4*(blockw+space)))

	for i := 0; i < blocks; i++ {
		warm := colorful.WarmColor()
		fwarm := colorful.FastWarmColor()
		happy := colorful.HappyColor()
		fhappy := colorful.FastHappyColor()
		draw.Draw(img, image.Rect(i*(blockw+space), 0, i*(blockw+space)+blockw, blockw), &image.Uniform{warm}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*(blockw+space), blockw+space, i*(blockw+space)+blockw, 2*blockw+space), &image.Uniform{fwarm}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*(blockw+space), 2*blockw+3*space, i*(blockw+space)+blockw, 3*blockw+3*space), &image.Uniform{happy}, image.Point{}, draw.Src)
		draw.Draw(img, image.Rect(i*(blockw+space), 3*blockw+4*space, i*(blockw+space)+blockw, 4*blockw+4*space), &image.Uniform{fhappy}, image.Point{}, draw.Src)
	}

	toimg, err := os.Create("colorgens.png")
	if err != nil {
		fmt.Printf("Error: %v", err)
		return
	}
	defer toimg.Close()

	png.Encode(toimg, img)
}
