#!/bin/sh
# Run tarpaulin and pycobertura to generate coverage.html.

cargo tarpaulin --skip-clean --out Xml
pycobertura show --format html --output coverage.html cobertura.xml
