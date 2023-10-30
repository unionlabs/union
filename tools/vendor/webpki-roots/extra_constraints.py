# -*- coding: utf-8 -*-
import binascii

# Agence Nationale de la Securite des Systemes d'Information (ANSSI)
ANSSI_SUBJECT_DN = (
    b"\x31\x0B\x30\x09\x06\x03\x55\x04\x06\x13\x02" b"FR"
    b"\x31\x0F\x30\x0D\x06\x03\x55\x04\x08\x13\x06" b"France"
    b"\x31\x0E\x30\x0C\x06\x03\x55\x04\x07\x13\x05" b"Paris"
    b"\x31\x10\x30\x0E\x06\x03\x55\x04\x0A\x13\x07" b"PM/SGDN"
    b"\x31\x0E\x30\x0C\x06\x03\x55\x04\x0B\x13\x05" b"DCSSI"
    b"\x31\x0E\x30\x0C\x06\x03\x55\x04\x03\x13\x05" b"IGC/A"
    b"\x31\x23\x30\x21\x06\x09\x2A\x86\x48\x86\xF7\x0D\x01\x09\x01"
    b"\x16\x14" b"igca@sgdn.pm.gouv.fr"
    )

ANSSI_NAME_CONSTRAINTS = (
    b"\xa0\x5f"
    b"\x30\x5D\xA0\x5B"
    b"\x30\x05\x82\x03" b".fr"
    b"\x30\x05\x82\x03" b".gp"
    b"\x30\x05\x82\x03" b".gf"
    b"\x30\x05\x82\x03" b".mq"
    b"\x30\x05\x82\x03" b".re"
    b"\x30\x05\x82\x03" b".yt"
    b"\x30\x05\x82\x03" b".pm"
    b"\x30\x05\x82\x03" b".bl"
    b"\x30\x05\x82\x03" b".mf"
    b"\x30\x05\x82\x03" b".wf"
    b"\x30\x05\x82\x03" b".pf"
    b"\x30\x05\x82\x03" b".nc"
    b"\x30\x05\x82\x03" b".tf"
    )

# TUBITAK Kamu SM SSL Kok Sertifikasi - Surum 1
TUBITAK1_SUBJECT_DN = (
    b"\x31\x0b\x30\x09\x06\x03\x55\x04\x06\x13\x02" b"TR"
    b"\x31\x18\x30\x16\x06\x03\x55\x04\x07\x13\x0f" b"Gebze - Kocaeli"
    b"\x31\x42\x30\x40\x06\x03\x55\x04\x0a\x13\x39" b"Turkiye Bilimsel ve Teknolojik Arastirma Kurumu - TUBITAK"
    b"\x31\x2d\x30\x2b\x06\x03\x55\x04\x0b\x13\x24" b"Kamu Sertifikasyon Merkezi - Kamu SM"
    b"\x31\x36\x30\x34\x06\x03\x55\x04\x03\x13\x2d" b"TUBITAK Kamu SM SSL Kok Sertifikasi - Surum 1"
    )

TUBITAK1_NAME_CONSTRAINTS = (
    b"\xa0\x67"
    b"\x30\x65\xa0\x63"
    b"\x30\x09\x82\x07" b".gov.tr"
    b"\x30\x09\x82\x07" b".k12.tr"
    b"\x30\x09\x82\x07" b".pol.tr"
    b"\x30\x09\x82\x07" b".mil.tr"
    b"\x30\x09\x82\x07" b".tsk.tr"
    b"\x30\x09\x82\x07" b".kep.tr"
    b"\x30\x09\x82\x07" b".bel.tr"
    b"\x30\x09\x82\x07" b".edu.tr"
    b"\x30\x09\x82\x07" b".org.tr"
    )

name_constraints = {
    TUBITAK1_SUBJECT_DN: TUBITAK1_NAME_CONSTRAINTS,
    ANSSI_SUBJECT_DN: ANSSI_NAME_CONSTRAINTS
}


def get_imposed_name_constraints(subject):
    """
    For the given certificate subject name, return a
    name constraints encoding which will be applied
    to that certificate.  Return None to apply
    no constraints.

    Data returned by this function is sourced from:

    https://hg.mozilla.org/projects/nss/file/tip/lib/certdb/genname.c

    Such that webpki-roots implements the same policy in this
    respect as the Mozilla root program.
    """

    return name_constraints.get(binascii.a2b_hex(subject), None)
