# Consensus Paper

The repository contains the specification (and the proofs) of the Tendermint
consensus protocol, adopted in CometBFT.

## How to install Latex on Mac OS

MacTex is Latex distribution for Mac OS. You can download it [here](http://www.tug.org/mactex/mactex-download.html).

Popular IDE for Latex-based projects is TexStudio. It can be downloaded
[here](https://www.texstudio.org/).

## How to build project

In order to compile the latex files (and write bibliography), execute

`$ pdflatex paper` <br/>
`$ bibtex paper` <br/>
`$ pdflatex paper` <br/>
`$ pdflatex paper` <br/>

The generated file is paper.pdf. You can open it with

`$ open paper.pdf`
