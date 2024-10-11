package empty_interface

//go:generate mockgen -package empty_interface -destination mock.go -source input.go -copyright_file=mock_copyright_header

type Empty interface{}
