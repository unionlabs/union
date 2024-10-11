## "sage sis.sage" will generate test_cases.json
## tested with a fresh sage install on macOS (Feb 2023)

import json

# BN254 Fr
r = 21888242871839275222246405745257275088548364400416034343698204186575808495617
frByteSize = 32
countToDeath = int(5)
gfr = GF(r)
Fr = GF(r)
Fr.<x> = Fr[]
rz = IntegerRing()

# Montgomery constant
rr = Fr(2**256)

# utils


def buildPoly(a):
    """ Builds a poly from the array a

    Args:
        a an array

    Returns:
        a[0]+a[1]*X + .. + a[n]*X**n
    """

    res = Fr(0)
    for i, v in enumerate(a):
        res += Fr(v)*x**i
    return res


def bitAt(i, b):
    """
    Args:
        i: index of the bit to retrieve
        b: array of bytes

    Returns:
        the i-th bit of b, when it is written b[0] || b[1] || ...
    """
    k = i//8
    if k >= len(b):
        return 0
    j = i % 8
    return (b[k] >> (7-j)) & 1


def toBytes(m, s):
    """

    Args:
        m: a bit int
        s: the expected number of bytes of the result. If s is bigger than the
        number of bytes in m, the remaining bytes are set to zero.

    Returns:
        the byte representation of m as a byte array, as
        in gnark-crypto.
    """
    _m = rz(m)
    res = s*[0]
    mask = 255
    for i in range(s):
        res[s-1-i] = _m & 255
        _m = _m >> 8
    return res


def splitCoeffs(b, logTwoBound):
    """
    Args:
        b: an array of bytes
        logTwoBound: number of bits of the bound

    Returns:
        an array of coeffs, each coeff being the i-th chunk of logTwoBounds bits of b.
        The coeffs are formed as follow. The input byte string is implicitly parsed as
        a slice of field elements of 32 bytes each in bigendian-natural form. the outputs
        are in a little-endian form. That is, each chunk of size 256 / logTwoBounds of the
        output can be seen as a polynomial, such that, when evaluated at 2 we get the original
        field element.
    """
    nbBits = len(b)*8
    res = [] 
    i = 0

    if len(b) % frByteSize != 0:
        exit("the length of b should divide the field size")

    # The number of fields that we are parsing. In case we have that
    # logTwoBound does not divide the number of bits to represent a
    # field element, we do not merge them.
    nbField = len(b) / 32
    nbBitsInField = int(frByteSize * 8)
    
    for fieldID in range(nbField):
        fieldStart = fieldID * 256
        e = 0
        for bitInField in range(nbBitsInField):
            j = bitInField % logTwoBound
            at = fieldStart + nbBitsInField - 1 - bitInField
            e |= bitAt(at, b) << j 
            # Switch to a new limb
            if j == logTwoBound - 1 or bitInField == frByteSize * 8 - 1:
                res.append(e)
                e = 0

    # careful Montgomery constant...
    return [Fr(e)*rr**-1 for e in res]


def polyRand(seed, n):
    """ Generates a pseudo random polynomial of size n from seed.

    Args:
        seed: seed for the pseudo random gen
        n: degree of the polynomial
    """
    seed = gfr(seed)
    a = n*[0]
    for i in range(n):
        a[i] = seed**2
        seed = a[i]
    return buildPoly(a)


# SIS
class SIS:
    def __init__(self, seed, logTwoDegree, logTwoBound, maxNbElementsToHash):
        """
            Args:
                seed
                logTwoDegree: 
                logTwoBound: bound of SIS
                maxNbElementsToHash
        """
        capacity = maxNbElementsToHash * frByteSize
        degree = 1 << logTwoDegree

        n = capacity * 8 / logTwoBound  # number of coefficients
        if n % degree == 0:  # check how sage / python rounds the int div.
            n = n / degree
        else:
            n = n / degree
            n = n + 1

        n = int(n)

        self.logTwoBound = logTwoBound
        self.degree = degree
        self.size = n
        self.key = n * [0]
        for i in range(n):
            self.key[i] = polyRand(seed, self.degree)
            seed += 1

    def hash(self, inputs):
        """ 
        Args:
           inputs is a vector of Fr elements

        Returns:
            the sis hash of m.
        """
        b = []
        for i in inputs:
            b.extend(toBytes(i, 32))

        return self.hash_bytes(b)

    def hash_bytes(self, b):
        """ 
        Args:
            b is a list of bytes to hash

        Returns:
            the sis hash of m.
        """
        # step 1: build the polynomials from m
        c = splitCoeffs(b, self.logTwoBound)
        mp = [buildPoly(c[self.degree*i:self.degree*(i+1)])
              for i in range(self.size)]

        # step 2: compute sum_i mp[i]*key[i] mod X^n+1
        modulo = x**self.degree+1
        res = 0
        for i in range(self.size):
            res += self.key[i]*mp[i]
        res = res % modulo
        return res


def vectorToString(v):
    # v is a vector of field elements
    # we return a list of strings in base10
    r = []
    for e in v:
        r.append("0x"+rz(e).hex())
    return r
    

def SISParams(seed, logTwoDegree, logTwoBound, maxNbElementsToHash):
    p = {}
    p['seed'] = int(seed)
    p['logTwoDegree'] = int(logTwoDegree)
    p['logTwoBound'] = int(logTwoBound)
    p['maxNbElementsToHash'] = int(maxNbElementsToHash)
    return p

params = [
    SISParams(5, 2, 3, 10),
    SISParams(5, 4, 3, 10),
    SISParams(5, 4, 4, 10),
    SISParams(5, 5, 4, 10),
    SISParams(5, 6, 5, 10),
    # SISParams(5, 8, 6, 10),
    SISParams(5, 10, 6, 10),
    SISParams(5, 11, 7, 10),
    SISParams(5, 12, 7, 10),
]

inputs = [
    [Fr(21888242871839275222246405745257275088548364400416034343698204186575808495614)],
    [Fr(1)],
    [Fr(42),Fr(8000)],
    [Fr(1),Fr(2), Fr(0),Fr(21888242871839275222246405745257275088548364400416034343698204186575808495616)],
    [Fr(1), Fr(0)],
    [Fr(0), Fr(1)],
    [Fr(0)],
    [Fr(0),Fr(0),Fr(0),Fr(0)],
    [Fr(0),Fr(0),Fr(8000),Fr(0)],
]

# sprinkle some random elements
for i in range(10):
    line = []
    for j in range(i):
        line.append(gfr.random_element())
    inputs.append(line)

testCases = {}
testCases['inputs'] = []
testCases['entries'] = []


for i, v in enumerate(inputs):
    testCases['inputs'].append(vectorToString(v))


for p in params:
    entry = {}
    entry['params'] = p
    entry['expected'] = []
    
    print("generating test cases with SIS params " + json.dumps(p))
    instance = SIS(p['seed'], p['logTwoDegree'], p['logTwoBound'], p['maxNbElementsToHash'])
    for i, v in enumerate(inputs):
        # hash the vector
        hResult = instance.hash(v)
        entry['expected'].append(vectorToString(hResult))
    
    testCases['entries'].append(entry)


testCases_json = json.dumps(testCases, indent=4)
with open("test_cases.json", "w") as outfile:
    outfile.write(testCases_json)
