import binascii
class InvalidEncodingException(Exception): pass
class NotOnCurveException(Exception): pass
class SpecException(Exception): pass

def lobit(x): return int(x) & 1
def hibit(x): return lobit(2*x)
def negative(x): return lobit(x)
def enc_le(x,n): return bytearray([int(x)>>(8*i) & 0xFF for i in xrange(n)])
def dec_le(x): return sum(b<<(8*i) for i,b in enumerate(x))
def randombytes(n): return bytearray([randint(0,255) for _ in range(n)])

def optimized_version_of(spec):
    """Decorator: This function is an optimized version of some specification"""
    def decorator(f):
        def wrapper(self,*args,**kwargs):
            def pr(x):
                if isinstance(x,bytearray): return binascii.hexlify(x)
                else: return str(x)
            try: spec_ans = getattr(self,spec,spec)(*args,**kwargs),None
            except Exception as e: spec_ans = None,e
            try: opt_ans = f(self,*args,**kwargs),None
            except Exception as e: opt_ans = None,e
            if spec_ans[1] is None and opt_ans[1] is not None:
                raise
                #raise SpecException("Mismatch in %s: spec returned %s but opt threw %s"
                #    % (f.__name__,str(spec_ans[0]),str(opt_ans[1])))
            if spec_ans[1] is not None and opt_ans[1] is None:
                raise
                #raise SpecException("Mismatch in %s: spec threw %s but opt returned %s"
                #    % (f.__name__,str(spec_ans[1]),str(opt_ans[0])))
            if spec_ans[0] != opt_ans[0]:
                raise SpecException("Mismatch in %s: %s != %s"
                    % (f.__name__,pr(spec_ans[0]),pr(opt_ans[0])))
            if opt_ans[1] is not None: raise
            else: return opt_ans[0]
        wrapper.__name__ = f.__name__
        return wrapper
    return decorator
    
def xsqrt(x,exn=InvalidEncodingException("Not on curve")):
    """Return sqrt(x)"""
    if not is_square(x): raise exn
    s = sqrt(x)
    if negative(s): s=-s
    return s        

def isqrt(x,exn=InvalidEncodingException("Not on curve")):
    """Return 1/sqrt(x)"""
    if x==0: return 0
    if not is_square(x): raise exn
    s = sqrt(x)
    #if negative(s): s=-s
    return 1/s

def inv0(x): return 1/x if x != 0 else 0

def isqrt_i(x):
    """Return 1/sqrt(x) or 1/sqrt(zeta * x)"""
    if x==0: return True,0
    gen = x.parent(-1)
    while is_square(gen): gen = sqrt(gen)
    if is_square(x): return True,1/sqrt(x)
    else: return False,1/sqrt(x*gen)

class QuotientEdwardsPoint(object):
    """Abstract class for point an a quotiented Edwards curve; needs F,a,d,cofactor to work"""
    def __init__(self,x=0,y=1):
        x = self.x = self.F(x)
        y = self.y = self.F(y)
        if y^2 + self.a*x^2 != 1 + self.d*x^2*y^2:
            raise NotOnCurveException(str(self))

    def __repr__(self):
        return "%s(0x%x,0x%x)" % (self.__class__.__name__, self.x, self.y)

    def __iter__(self):
        yield self.x
        yield self.y

    def __add__(self,other):
        x,y = self
        X,Y = other
        a,d = self.a,self.d
        return self.__class__(
            (x*Y+y*X)/(1+d*x*y*X*Y), 
            (y*Y-a*x*X)/(1-d*x*y*X*Y)
        )
    
    def __neg__(self): return self.__class__(-self.x,self.y)
    def __sub__(self,other): return self + (-other)
    def __rmul__(self,other): return self*other
    def __eq__(self,other):
        """NB: this is the only method that is different from the usual one"""
        x,y = self
        X,Y = other
        return x*Y == X*y or (self.cofactor==8 and -self.a*x*X == y*Y)
    def __ne__(self,other): return not (self==other)
    
    def __mul__(self,exp):
        exp = int(exp)
        if exp < 0: exp,self = -exp,-self
        total = self.__class__()
        work  = self
        while exp != 0:
            if exp & 1: total += work
            work += work
            exp >>= 1
        return total
    
    def xyzt(self):
        x,y = self
        z = self.F.random_element()
        return x*z,y*z,z,x*y*z
        
    def torque(self):
        """Apply cofactor group, except keeping the point even"""
        if self.cofactor == 8:
            if self.a == -1: return self.__class__(self.y*self.i, self.x*self.i)
            if self.a ==  1: return self.__class__(-self.y, self.x)
        else:
            return self.__class__(-self.x, -self.y)
    
    def doubleAndEncodeSpec(self):
        return (self+self).encode()

    # Utility functions
    @classmethod
    def bytesToGf(cls,bytes,mustBeProper=True,mustBePositive=False,maskHiBits=False):
        """Convert little-endian bytes to field element, sanity check length"""
        if len(bytes) != cls.encLen:
            raise InvalidEncodingException("wrong length %d" % len(bytes))
        s = dec_le(bytes)
        if mustBeProper and s >= cls.F.order():
            raise InvalidEncodingException("%d out of range!" % s)
        bitlen = int(ceil(log(cls.F.order())/log(2)))
        if maskHiBits: s &= 2^bitlen-1
        s = cls.F(s)
        if mustBePositive and negative(s):
            raise InvalidEncodingException("%d is negative!" % s)
        return s
        
    @classmethod
    def gfToBytes(cls,x,mustBePositive=False):
        """Convert little-endian bytes to field element, sanity check length"""
        if negative(x) and mustBePositive: x = -x
        return enc_le(x,cls.encLen)

class RistrettoPoint(QuotientEdwardsPoint):
    """The new Ristretto group"""
    def encodeSpec(self):
        """Unoptimized specification for encoding"""
        x,y = self
        if self.cofactor==8 and (negative(x*y) or y==0): (x,y) = self.torque()
        if y == -1: y = 1 # Avoid divide by 0; doesn't affect impl
            
        if negative(x): x,y = -x,-y
        s = xsqrt(self.mneg*(1-y)/(1+y),exn=Exception("Unimplemented: point is odd: " + str(self)))
        return self.gfToBytes(s)
        
    @classmethod
    def decodeSpec(cls,s):
        """Unoptimized specification for decoding"""
        s = cls.bytesToGf(s,mustBePositive=True)
        
        a,d = cls.a,cls.d
        x = xsqrt(4*s^2 / (a*d*(1+a*s^2)^2 - (1-a*s^2)^2))
        y = (1+a*s^2) / (1-a*s^2)
    
        if cls.cofactor==8 and (negative(x*y) or y==0):
            raise InvalidEncodingException("x*y has high bit")
                
        return cls(x,y)

    @optimized_version_of("encodeSpec")
    def encode(self):
        """Encode, optimized version"""
        a,d,mneg = self.a,self.d,self.mneg
        x,y,z,t = self.xyzt()
        
        if self.cofactor==8:
            u1    = mneg*(z+y)*(z-y)
            u2    = x*y # = t*z
            isr   = isqrt(u1*u2^2)
            i1    = isr*u1 # sqrt(mneg*(z+y)*(z-y))/(x*y)
            i2    = isr*u2 # 1/sqrt(a*(y+z)*(y-z))
            z_inv = i1*i2*t # 1/z
        
            if negative(t*z_inv):
                if a==-1:
                    x,y = y*self.i,x*self.i
                    den_inv = self.magic * i1
                else:
                    x,y = -y,x
                    den_inv = self.i * self.magic * i1
                
            else:
                den_inv = i2

            if negative(x*z_inv): y = -y
            s = (z-y) * den_inv
        else:
            num   = mneg*(z+y)*(z-y)
            isr   = isqrt(num*y^2)
            if negative(isr^2*num*y*t): y = -y
            s = isr*y*(z-y)
            
        return self.gfToBytes(s,mustBePositive=True)
      
    @optimized_version_of("doubleAndEncodeSpec")
    def doubleAndEncode(self):
        X,Y,Z,T = self.xyzt()
        a,d,mneg = self.a,self.d,self.mneg

        if self.cofactor==8:
            e = 2*X*Y
            f = Z^2+d*T^2
            g = Y^2-a*X^2
            h = Z^2-d*T^2
            
            inv1 = 1/(e*f*g*h)
            z_inv = inv1*e*g # 1 / (f*h)
            t_inv = inv1*f*h
        
            if negative(e*g*z_inv):
                if a==-1: sqrta = self.i
                else:     sqrta = -1
                e,f,g,h = g,h,-e,f*sqrta
                factor = self.i
            else:
                factor = self.magic
            
            if negative(h*e*z_inv): g=-g
            s = (h-g)*factor*g*t_inv
            
        else:
            foo = Y^2+a*X^2
            bar = X*Y
            den = 1/(foo*bar)
            if negative(2*bar^2*den): tmp = a*X^2
            else: tmp = Y^2
            s = self.magic*(Z^2-tmp)*foo*den
            
        return self.gfToBytes(s,mustBePositive=True)
        
    @classmethod
    @optimized_version_of("decodeSpec")
    def decode(cls,s):
        """Decode, optimized version"""
        s = cls.bytesToGf(s,mustBePositive=True)
        
        a,d = cls.a,cls.d
        yden     = 1-a*s^2
        ynum     = 1+a*s^2
        yden_sqr = yden^2
        xden_sqr = a*d*ynum^2 - yden_sqr
        
        isr = isqrt(xden_sqr * yden_sqr)
        
        xden_inv = isr * yden
        yden_inv = xden_inv * isr * xden_sqr
        
        x = 2*s*xden_inv
        if negative(x): x = -x
        y = ynum * yden_inv
    
        if cls.cofactor==8 and (negative(x*y) or y==0):
            raise InvalidEncodingException("x*y is invalid: %d, %d" % (x,y))
            
        return cls(x,y)
       
    @classmethod     
    def fromJacobiQuartic(cls,s,t,sgn=1):
        """Convert point from its Jacobi Quartic representation"""
        a,d = cls.a,cls.d
        assert s^4 - 2*cls.a*(1-2*d/(d-a))*s^2 + 1 == t^2
        x = 2*s*cls.magic / t
        y = (1+a*s^2) / (1-a*s^2)
        return cls(sgn*x,y)
            
    @classmethod
    def elligatorSpec(cls,r0):
        a,d = cls.a,cls.d
        r = cls.qnr * cls.bytesToGf(r0,mustBeProper=False,maskHiBits=True)^2
        den = (d*r-a)*(a*r-d)
        if den == 0: return cls()
        n1 = cls.a*(r+1)*(a+d)*(d-a)/den
        n2 = r*n1
        if is_square(n1):
            sgn,s,t =  1, xsqrt(n1), -(r-1)*(a+d)^2 / den - 1
        else:
            sgn,s,t = -1,-xsqrt(n2), r*(r-1)*(a+d)^2 / den - 1
        
        return cls.fromJacobiQuartic(s,t)
            
    @classmethod
    @optimized_version_of("elligatorSpec")
    def elligator(cls,r0):
        a,d = cls.a,cls.d
        r0 = cls.bytesToGf(r0,mustBeProper=False,maskHiBits=True)
        r = cls.qnr * r0^2
        den = (d*r-a)*(a*r-d)
        num = cls.a*(r+1)*(a+d)*(d-a)
        
        iss,isri = isqrt_i(num*den)
        if iss: sgn,twiddle =  1,1
        else:   sgn,twiddle = -1,r0*cls.qnr
        isri *= twiddle
        s = isri*num
        t = -sgn*isri*s*(r-1)*(d+a)^2 - 1
        if negative(s) == iss: s = -s
        return cls.fromJacobiQuartic(s,t)


class Decaf_1_1_Point(QuotientEdwardsPoint):
    """Like current decaf but tweaked for simplicity"""
    def encodeSpec(self):
        """Unoptimized specification for encoding"""
        a,d = self.a,self.d
        x,y = self
        if x==0 or y==0: return(self.gfToBytes(0))
        
        if self.cofactor==8 and negative(x*y*self.isoMagic):
            x,y = self.torque()
            
        sr = xsqrt(1-a*x^2)
        altx = x*y*self.isoMagic / sr
        if negative(altx): s = (1+sr)/x
        else:              s = (1-sr)/x
        
        return self.gfToBytes(s,mustBePositive=True)
        
    @classmethod
    def decodeSpec(cls,s):
        """Unoptimized specification for decoding"""
        a,d = cls.a,cls.d
        s = cls.bytesToGf(s,mustBePositive=True)
        
        if s==0: return cls()
        t = xsqrt(s^4 + 2*(a-2*d)*s^2 + 1)
        altx = 2*s*cls.isoMagic/t
        if negative(altx): t = -t
        x = 2*s / (1+a*s^2)
        y = (1-a*s^2) / t
        
        if cls.cofactor==8 and (negative(x*y*cls.isoMagic) or y==0):
            raise InvalidEncodingException("x*y is invalid: %d, %d" % (x,y))
        
        return cls(x,y)

    def toJacobiQuartic(self,toggle_rotation=False,toggle_altx=False,toggle_s=False):
        "Return s,t on jacobi curve"
        a,d = self.a,self.d
        x,y,z,t = self.xyzt()
        
        if self.cofactor == 8:
            # Cofactor 8 version
            # Simulate IMAGINE_TWIST because that's how libdecaf does it
            x = self.i*x
            t = self.i*t
            a = -a
            d = -d
            
            # OK, the actual libdecaf code should be here
            num = (z+y)*(z-y)
            den = x*y
            isr = isqrt(num*(a-d)*den^2)
    
            iden = isr * den * self.isoMagic # 1/sqrt((z+y)(z-y)) = 1/sqrt(1-Y^2) / z
            inum = isr * num # sqrt(1-Y^2) * z / xysqrt(a-d) ~ 1/sqrt(1-ax^2)/z
            
            if negative(iden*inum*self.i*t^2*(d-a)) != toggle_rotation:
                iden,inum = inum,iden
                fac = x*sqrt(a)
                toggle=(a==-1)
            else:
                fac = y
                toggle=False
            
            imi = self.isoMagic * self.i
            altx = inum*t*imi
            neg_altx = negative(altx) != toggle_altx
            if neg_altx != toggle: inum =- inum
            
            tmp = fac*(inum*z + 1)
            s = iden*tmp*imi
            
            negm1 = (negative(s) != toggle_s) != neg_altx
            if negm1: m1 = a*fac + z
            else:     m1 = a*fac - z
            
            swap = toggle_s
        
        else:
            # Much simpler cofactor 4 version
            num = (x+t)*(x-t)
            isr = isqrt(num*(a-d)*x^2)
            ratio = isr*num 
            altx = ratio*self.isoMagic
            
            neg_altx = negative(altx) != toggle_altx
            if neg_altx: ratio =- ratio
                
            tmp = ratio*z - t
            s = (a-d)*isr*x*tmp
            
            negx = (negative(s) != toggle_s) != neg_altx
            if negx: m1 = -a*t + x
            else:    m1 = -a*t - x
            
            swap = toggle_s
            
        if negative(s): s = -s
        
        return s,m1,a*tmp,swap
    
    def invertElligator(self,toggle_r=False,*args,**kwargs):
        "Produce preimage of self under elligator, or None"
        a,d = self.a,self.d
        
        rets = []
        
        tr = [False,True] if self.cofactor == 8 else [False]
        for toggle_rotation in tr:
            for toggle_altx in [False,True]:
                for toggle_s in [False,True]:
                    for toggle_r in [False,True]:
                        s,m1,m12,swap = self.toJacobiQuartic(toggle_rotation,toggle_altx,toggle_s)

                        #print
                        #print toggle_rotation,toggle_altx,toggle_s
                        #print m1
                        #print m12
                    
                    
                        if self == self.__class__():
                            if self.cofactor == 4:
                                # Hacks for identity!
                                if toggle_altx: m12 = 1
                                elif toggle_s: m1 = 1
                                elif toggle_r: continue
                                ## BOTH???
                                
                            else:
                                m12 = 1
                                imi = self.isoMagic * self.i
                                if toggle_rotation:
                                    if toggle_altx: m1 = -imi
                                    else:           m1 = +imi
                                else:
                                    if toggle_altx: m1 = 0
                                    else: m1 = a-d
                    
                        rnum = (d*a*m12-m1)
                        rden = ((d*a-1)*m12+m1)
                        if swap: rnum,rden = rden,rnum
                    
                        ok,sr = isqrt_i(rnum*rden*self.qnr)
                        if not ok: continue
                        sr *= rnum
                        #print "Works! %d %x" % (swap,sr)
                    
                        if negative(sr) != toggle_r: sr = -sr
                        ret = self.gfToBytes(sr)
                        if self.elligator(ret) != self and self.elligator(ret) != -self:
                            print "WRONG!",[toggle_rotation,toggle_altx,toggle_s]
                        if self.elligator(ret) == -self and self != -self: print "Negated!",[toggle_rotation,toggle_altx,toggle_s]
                        rets.append(bytes(ret))
        return rets

    @optimized_version_of("encodeSpec")
    def encode(self):
        """Encode, optimized version"""    
        return self.gfToBytes(self.toJacobiQuartic()[0])
        
    @classmethod
    @optimized_version_of("decodeSpec")
    def decode(cls,s):
        """Decode, optimized version"""
        a,d = cls.a,cls.d
        s = cls.bytesToGf(s,mustBePositive=True)
        
        #if s==0: return cls()
        s2 = s^2
        den = 1+a*s2
        num = den^2 - 4*d*s2
        isr = isqrt(num*den^2)
        altx = 2*s*isr*den*cls.isoMagic
        if negative(altx): isr = -isr
        x = 2*s *isr^2*den*num
        y = (1-a*s^2) * isr*den
        
        if cls.cofactor==8 and (negative(x*y*cls.isoMagic) or y==0):
            raise InvalidEncodingException("x*y is invalid: %d, %d" % (x,y))
        
        return cls(x,y)

    @classmethod     
    def fromJacobiQuartic(cls,s,t,sgn=1):
        """Convert point from its Jacobi Quartic representation"""
        a,d = cls.a,cls.d
        if s==0: return cls()
        x = 2*s / (1+a*s^2)
        y = (1-a*s^2) / t
        return cls(x,sgn*y)

    @optimized_version_of("doubleAndEncodeSpec")
    def doubleAndEncode(self):
        X,Y,Z,T = self.xyzt()
        a,d = self.a,self.d
        
        if self.cofactor == 8:
            # Cofactor 8 version
            # Simulate IMAGINE_TWIST because that's how libdecaf does it
            X = self.i*X
            T = self.i*T
            a = -a
            d = -d
            # TODO: This is only being called for a=-1, so could
            # be wrong for a=1
            
            e = 2*X*Y
            f = Y^2+a*X^2
            g = Y^2-a*X^2
            h = Z^2-d*T^2
            
            eim = e*self.isoMagic
            inv = 1/(eim*g*f*h)
            fh_inv = eim*g*inv*self.i
            
            if negative(eim*g*fh_inv):
                idf = g*self.isoMagic*self.i
                bar = f
                foo = g
                test = eim*f
            else:
                idf = eim
                bar = h
                foo = -eim
                test = g*h
            
            if negative(test*fh_inv): bar =- bar
            s = idf*(foo+bar)*inv*f*h
        
        else:
            xy = X*Y
            h = Z^2-d*T^2
            inv = 1/(xy*h)
            if negative(inv*2*xy^2*self.isoMagic): tmp = Y
            else: tmp = X
            s = tmp^2*h*inv # = X/Y or Y/X, interestingly
            
        return self.gfToBytes(s,mustBePositive=True)
            
    @classmethod
    def elligatorSpec(cls,r0,fromR=False):
        a,d = cls.a,cls.d
        if fromR: r = r0
        else: r = cls.qnr * cls.bytesToGf(r0,mustBeProper=False,maskHiBits=True)^2
        
        den = (d*r-(d-a))*((d-a)*r-d)
        if den == 0: return cls()
        n1 = (r+1)*(a-2*d)/den
        n2 = r*n1
        if is_square(n1):
            sgn,s,t = 1,   xsqrt(n1),  -(r-1)*(a-2*d)^2 / den - 1
        else:
            sgn,s,t = -1, -xsqrt(n2), r*(r-1)*(a-2*d)^2 / den - 1
        
        return cls.fromJacobiQuartic(s,t)
            
    @classmethod
    @optimized_version_of("elligatorSpec")
    def elligator(cls,r0):
        a,d = cls.a,cls.d
        r0 = cls.bytesToGf(r0,mustBeProper=False,maskHiBits=True)
        r = cls.qnr * r0^2
        den = (d*r-(d-a))*((d-a)*r-d)
        num = (r+1)*(a-2*d)
        
        iss,isri = isqrt_i(num*den)
        if iss: sgn,twiddle =  1,1
        else:   sgn,twiddle = -1,r0*cls.qnr
        isri *= twiddle
        s = isri*num
        t = -sgn*isri*s*(r-1)*(a-2*d)^2 - 1
        if negative(s) == iss: s = -s
        return cls.fromJacobiQuartic(s,t)
            
    def elligatorInverseBruteForce(self):
        """Invert Elligator using SAGE's polynomial solver"""
        a,d = self.a,self.d
        R.<r0> = self.F[]
        r = self.qnr * r0^2
        den = (d*r-(d-a))*((d-a)*r-d)
        n1 = (r+1)*(a-2*d)/den
        n2 = r*n1
        ret = set()
        for s2,t in [(n1, -(r-1)*(a-2*d)^2 / den - 1),
                     (n2,r*(r-1)*(a-2*d)^2 / den - 1)]:
            x2 = 4*s2/(1+a*s2)^2
            y = (1-a*s2) / t

            selfT = self
            for i in xrange(self.cofactor/2):
                xT,yT = selfT
                polyX = xT^2-x2
                polyY = yT-y
                sx = set(r for r,_ in polyX.numerator().roots())
                sy = set(r for r,_ in polyY.numerator().roots())
                ret = ret.union(sx.intersection(sy))
            
                selfT = selfT.torque()

        ret = [self.gfToBytes(r) for r in ret]
        
        for r in ret:
            assert self.elligator(r) in [self,-self]
            
        ret = [r for r in ret if self.elligator(r) == self]

        return ret
            
class Ed25519Point(RistrettoPoint):
    F = GF(2^255-19)
    d = F(-121665/121666)
    a = F(-1)
    i = sqrt(F(-1))
    mneg = F(1)
    qnr = i
    magic = isqrt(a*d-1)
    cofactor = 8
    encLen = 32
    
    @classmethod
    def base(cls):
        return cls( 15112221349535400772501151409588531511454012693041857206046113283949847762202, 46316835694926478169428394003475163141307993866256225615783033603165251855960
        )
            
class NegEd25519Point(RistrettoPoint):
    F = GF(2^255-19)
    d = F(121665/121666)
    a = F(1)
    i = sqrt(F(-1))
    mneg = F(-1) # TODO checkme vs 1-ad or whatever
    qnr = i
    magic = isqrt(a*d-1)
    cofactor = 8
    encLen = 32
    
    @classmethod
    def base(cls):
        y = cls.F(4/5)
        x = sqrt((y^2-1)/(cls.d*y^2-cls.a))
        if negative(x): x = -x
        return cls(x,y)

class IsoEd448Point(RistrettoPoint):
    F = GF(2^448-2^224-1)
    d = F(39082/39081)
    a = F(1)
    mneg = F(-1)
    qnr = -1
    magic = isqrt(a*d-1)
    cofactor = 4
    encLen = 56
    
    @classmethod
    def base(cls):
        return cls(  # RFC has it wrong
         345397493039729516374008604150537410266655260075183290216406970281645695073672344430481787759340633221708391583424041788924124567700732,
            -363419362147803445274661903944002267176820680343659030140745099590306164083365386343198191849338272965044442230921818680526749009182718
        )
            
class TwistedEd448GoldilocksPoint(Decaf_1_1_Point):
    F = GF(2^448-2^224-1)
    d = F(-39082)
    a = F(-1)
    qnr = -1
    cofactor = 4
    encLen = 56
    isoMagic = IsoEd448Point.magic

    @classmethod
    def base(cls):
        return cls.decodeSpec(Ed448GoldilocksPoint.base().encodeSpec())

class Ed448GoldilocksPoint(Decaf_1_1_Point):
    F = GF(2^448-2^224-1)
    d = F(-39081)
    a = F(1)
    qnr = -1
    cofactor = 4
    encLen = 56
    isoMagic = IsoEd448Point.magic
    
    @classmethod
    def base(cls):
        return 2*cls(
 224580040295924300187604334099896036246789641632564134246125461686950415467406032909029192869357953282578032075146446173674602635247710, 298819210078481492676017930443930673437544040154080242095928241372331506189835876003536878655418784733982303233503462500531545062832660
        )

class IsoEd25519Point(Decaf_1_1_Point):
    # TODO: twisted iso too!
    # TODO: twisted iso might have to IMAGINE_TWIST or whatever
    F = GF(2^255-19)
    d = F(-121665)
    a = F(1)
    i = sqrt(F(-1))
    qnr = i
    magic = isqrt(a*d-1)
    cofactor = 8
    encLen = 32
    isoMagic = Ed25519Point.magic
    isoA = Ed25519Point.a
    
    @classmethod
    def base(cls):
        return cls.decodeSpec(Ed25519Point.base().encode())

class TestFailedException(Exception): pass

def test(cls,n):
    print "Testing curve %s" % cls.__name__
    
    specials = [1]
    ii = cls.F(-1)
    while is_square(ii):
        specials.append(ii)
        ii = sqrt(ii)
    specials.append(ii)
    for i in specials:
        if negative(cls.F(i)): i = -i
        i = enc_le(i,cls.encLen)
        try:
            Q = cls.decode(i)
            QE = Q.encode()
            if QE != i:
                raise TestFailedException("Round trip special %s != %s" %
                     (binascii.hexlify(QE),binascii.hexlify(i)))
        except NotOnCurveException: pass
        except InvalidEncodingException: pass
        
    
    P = cls.base()
    Q = cls()
    for i in xrange(n):
        #print binascii.hexlify(Q.encode())
        QE = Q.encode()
        QQ = cls.decode(QE)
        if QQ != Q: raise TestFailedException("Round trip %s != %s" % (str(QQ),str(Q)))
    
        # Testing s -> 1/s: encodes -point on cofactor 
        s = cls.bytesToGf(QE)
        if s != 0:
            ss = cls.gfToBytes(1/s,mustBePositive=True)
            try:
                QN = cls.decode(ss)
                if cls.cofactor == 8:
                    raise TestFailedException("1/s shouldnt work for cofactor 8")
                if QN != -Q:
                    raise TestFailedException("s -> 1/s should negate point for cofactor 4")
            except InvalidEncodingException as e:
                # Should be raised iff cofactor==8
                if cls.cofactor == 4:
                    raise TestFailedException("s -> 1/s should work for cofactor 4")
        
        QT = Q
        for h in xrange(cls.cofactor):
            QT = QT.torque()
            if QT.encode() != QE:
                raise TestFailedException("Can't torque %s,%d" % (str(Q),h+1))
            
        Q0 = Q + P
        if Q0 == Q: raise TestFailedException("Addition doesn't work")
        if Q0-P != Q: raise TestFailedException("Subtraction doesn't work")
        
        r = randint(1,1000)
        Q1 = Q0*r
        Q2 = Q0*(r+1)
        if Q1 + Q0 != Q2: raise TestFailedException("Scalarmul doesn't work")
        Q = Q1
        
def testElligator(cls,n):
    print "Testing elligator on %s" % cls.__name__
    for i in xrange(n):
        r = randombytes(cls.encLen)
        P = cls.elligator(r)
        if hasattr(P,"invertElligator"):
            iv = P.invertElligator()
            modr = bytes(cls.gfToBytes(cls.bytesToGf(r,mustBeProper=False,maskHiBits=True)))
            iv2 = P.torque().invertElligator()
            if modr not in iv: print "Failed to invert Elligator!"
            if len(iv) != len(set(iv)):
                print "Elligator inverses not unique!", len(set(iv)), len(iv)
            if iv != iv2:
                print "Elligator is untorqueable!"
                #print [binascii.hexlify(j) for j in iv]
                #print [binascii.hexlify(j) for j in iv2]
                #break
        else:
            pass # TODO

def gangtest(classes,n):
    print "Gang test",[cls.__name__ for cls in classes]
    specials = [1]
    ii = classes[0].F(-1)
    while is_square(ii):
        specials.append(ii)
        ii = sqrt(ii)
    specials.append(ii)
    
    for i in xrange(n):
        rets = [bytes((cls.base()*i).encode()) for cls in classes]
        if len(set(rets)) != 1:
            print "Divergence in encode at %d" % i
            for c,ret in zip(classes,rets):
                print c,binascii.hexlify(ret)
            print
        
        if i < len(specials): r0 = enc_le(specials[i],classes[0].encLen)
        else: r0 = randombytes(classes[0].encLen)
        
        rets = [bytes((cls.elligator(r0)*i).encode()) for cls in classes]
        if len(set(rets)) != 1:
            print "Divergence in elligator at %d" % i
            for c,ret in zip(classes,rets):
                print c,binascii.hexlify(ret)
            print

def testDoubleAndEncode(cls,n):
    print "Testing doubleAndEncode on %s" % cls.__name__
    for i in xrange(n):
        r1 = randombytes(cls.encLen)
        r2 = randombytes(cls.encLen)
        u = cls.elligator(r1) + cls.elligator(r2)
        u.doubleAndEncode()

testDoubleAndEncode(Ed25519Point,100)
testDoubleAndEncode(NegEd25519Point,100)
testDoubleAndEncode(IsoEd25519Point,100)
testDoubleAndEncode(IsoEd448Point,100)
testDoubleAndEncode(TwistedEd448GoldilocksPoint,100)
#test(Ed25519Point,100)
#test(NegEd25519Point,100)
#test(IsoEd25519Point,100)
#test(IsoEd448Point,100)
#test(TwistedEd448GoldilocksPoint,100)
#test(Ed448GoldilocksPoint,100)
#testElligator(Ed25519Point,100)
#testElligator(NegEd25519Point,100)
#testElligator(IsoEd25519Point,100)
#testElligator(IsoEd448Point,100)
#testElligator(Ed448GoldilocksPoint,100)
#testElligator(TwistedEd448GoldilocksPoint,100)
#gangtest([IsoEd448Point,TwistedEd448GoldilocksPoint,Ed448GoldilocksPoint],100)
#gangtest([Ed25519Point,IsoEd25519Point],100)
