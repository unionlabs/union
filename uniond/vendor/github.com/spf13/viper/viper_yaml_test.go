package viper

var yamlExample = []byte(`Hacker: true
name: steve
hobbies:
    - skateboarding
    - snowboarding
    - go
clothing:
    jacket: leather
    trousers: denim
    pants:
        size: large
age: 35
eyes : brown
beard: true
`)

var yamlWriteExpected = []byte(`age: 35
beard: true
clothing:
    jacket: leather
    pants:
        size: large
    trousers: denim
eyes: brown
hacker: true
hobbies:
    - skateboarding
    - snowboarding
    - go
name: steve
`)

var yamlExampleWithDot = []byte(`Hacker: true
name: steve
hobbies:
    - skateboarding
    - snowboarding
    - go
clothing:
    jacket: leather
    trousers: denim
    pants:
        size: large
age: 35
eyes : brown
beard: true
emails:
    steve@hacker.com:
        created: 01/02/03
        active: true
`)
