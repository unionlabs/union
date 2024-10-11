package empty_interface

//go:generate mockgen -package empty_interface -destination mock.go -source input.go

type Empty interface{}
