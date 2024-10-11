//go:build windows
// +build windows

package backuptar

import (
	"archive/tar"
	"encoding/base64"
	"fmt"
	"io"
	"path/filepath"
	"strconv"
	"strings"
	"syscall"
	"time"

	"github.com/Microsoft/go-winio"
	"golang.org/x/sys/windows"
)

//nolint:deadcode,varcheck // keep unused constants for potential future use
const (
	cISUID  = 0004000 // Set uid
	cISGID  = 0002000 // Set gid
	cISVTX  = 0001000 // Save text (sticky bit)
	cISDIR  = 0040000 // Directory
	cISFIFO = 0010000 // FIFO
	cISREG  = 0100000 // Regular file
	cISLNK  = 0120000 // Symbolic link
	cISBLK  = 0060000 // Block special file
	cISCHR  = 0020000 // Character special file
	cISSOCK = 0140000 // Socket
)

const (
	hdrFileAttributes        = "MSWINDOWS.fileattr"
	hdrSecurityDescriptor    = "MSWINDOWS.sd"
	hdrRawSecurityDescriptor = "MSWINDOWS.rawsd"
	hdrMountPoint            = "MSWINDOWS.mountpoint"
	hdrEaPrefix              = "MSWINDOWS.xattr."

	hdrCreationTime = "LIBARCHIVE.creationtime"
)

// zeroReader is an io.Reader that always returns 0s.
type zeroReader struct{}

func (zeroReader) Read(b []byte) (int, error) {
	for i := range b {
		b[i] = 0
	}
	return len(b), nil
}

func copySparse(t *tar.Writer, br *winio.BackupStreamReader) error {
	curOffset := int64(0)
	for {
		bhdr, err := br.Next()
		if err == io.EOF { //nolint:errorlint
			err = io.ErrUnexpectedEOF
		}
		if err != nil {
			return err
		}
		if bhdr.Id != winio.BackupSparseBlock {
			return fmt.Errorf("unexpected stream %d", bhdr.Id)
		}

		// We can't seek backwards, since we have already written that data to the tar.Writer.
		if bhdr.Offset < curOffset {
			return fmt.Errorf("cannot seek back from %d to %d", curOffset, bhdr.Offset)
		}
		// archive/tar does not support writing sparse files
		// so just write zeroes to catch up to the current offset.
		if _, err = io.CopyN(t, zeroReader{}, bhdr.Offset-curOffset); err != nil {
			return fmt.Errorf("seek to offset %d: %w", bhdr.Offset, err)
		}
		if bhdr.Size == 0 {
			// A sparse block with size = 0 is used to mark the end of the sparse blocks.
			break
		}
		n, err := io.Copy(t, br)
		if err != nil {
			return err
		}
		if n != bhdr.Size {
			return fmt.Errorf("copied %d bytes instead of %d at offset %d", n, bhdr.Size, bhdr.Offset)
		}
		curOffset = bhdr.Offset + n
	}
	return nil
}

// BasicInfoHeader creates a tar header from basic file information.
func BasicInfoHeader(name string, size int64, fileInfo *winio.FileBasicInfo) *tar.Header {
	hdr := &tar.Header{
		Format:     tar.FormatPAX,
		Name:       filepath.ToSlash(name),
		Size:       size,
		Typeflag:   tar.TypeReg,
		ModTime:    time.Unix(0, fileInfo.LastWriteTime.Nanoseconds()),
		ChangeTime: time.Unix(0, fileInfo.ChangeTime.Nanoseconds()),
		AccessTime: time.Unix(0, fileInfo.LastAccessTime.Nanoseconds()),
		PAXRecords: make(map[string]string),
	}
	hdr.PAXRecords[hdrFileAttributes] = fmt.Sprintf("%d", fileInfo.FileAttributes)
	hdr.PAXRecords[hdrCreationTime] = formatPAXTime(time.Unix(0, fileInfo.CreationTime.Nanoseconds()))

	if (fileInfo.FileAttributes & syscall.FILE_ATTRIBUTE_DIRECTORY) != 0 {
		hdr.Mode |= cISDIR
		hdr.Size = 0
		hdr.Typeflag = tar.TypeDir
	}
	return hdr
}

// SecurityDescriptorFromTarHeader reads the SDDL associated with the header of the current file
// from the tar header and returns the security descriptor into a byte slice.
func SecurityDescriptorFromTarHeader(hdr *tar.Header) ([]byte, error) {
	if sdraw, ok := hdr.PAXRecords[hdrRawSecurityDescriptor]; ok {
		sd, err := base64.StdEncoding.DecodeString(sdraw)
		if err != nil {
			// Not returning sd as-is in the error-case, as base64.DecodeString
			// may return partially decoded data (not nil or empty slice) in case
			// of a failure: https://github.com/golang/go/blob/go1.17.7/src/encoding/base64/base64.go#L382-L387
			return nil, err
		}
		return sd, nil
	}
	// Maintaining old SDDL-based behavior for backward compatibility. All new
	// tar headers written by this library will have raw binary for the security
	// descriptor.
	if sddl, ok := hdr.PAXRecords[hdrSecurityDescriptor]; ok {
		return winio.SddlToSecurityDescriptor(sddl)
	}
	return nil, nil
}

// ExtendedAttributesFromTarHeader reads the EAs associated with the header of the
// current file from the tar header and returns it as a byte slice.
func ExtendedAttributesFromTarHeader(hdr *tar.Header) ([]byte, error) {
	var eas []winio.ExtendedAttribute //nolint:prealloc // len(eas) <= len(hdr.PAXRecords); prealloc is wasteful
	for k, v := range hdr.PAXRecords {
		if !strings.HasPrefix(k, hdrEaPrefix) {
			continue
		}
		data, err := base64.StdEncoding.DecodeString(v)
		if err != nil {
			return nil, err
		}
		eas = append(eas, winio.ExtendedAttribute{
			Name:  k[len(hdrEaPrefix):],
			Value: data,
		})
	}
	var eaData []byte
	var err error
	if len(eas) != 0 {
		eaData, err = winio.EncodeExtendedAttributes(eas)
		if err != nil {
			return nil, err
		}
	}
	return eaData, nil
}

// EncodeReparsePointFromTarHeader reads the ReparsePoint structure from the tar header
// and encodes it into a byte slice. The file for which this function is called must be a
// symlink.
func EncodeReparsePointFromTarHeader(hdr *tar.Header) []byte {
	_, isMountPoint := hdr.PAXRecords[hdrMountPoint]
	rp := winio.ReparsePoint{
		Target:       filepath.FromSlash(hdr.Linkname),
		IsMountPoint: isMountPoint,
	}
	return winio.EncodeReparsePoint(&rp)
}

// WriteTarFileFromBackupStream writes a file to a tar writer using data from a Win32 backup stream.
//
// This encodes Win32 metadata as tar pax vendor extensions starting with MSWINDOWS.
//
// The additional Win32 metadata is:
//
//   - MSWINDOWS.fileattr: The Win32 file attributes, as a decimal value
//   - MSWINDOWS.rawsd: The Win32 security descriptor, in raw binary format
//   - MSWINDOWS.mountpoint: If present, this is a mount point and not a symlink, even though the type is '2' (symlink)
func WriteTarFileFromBackupStream(t *tar.Writer, r io.Reader, name string, size int64, fileInfo *winio.FileBasicInfo) error {
	name = filepath.ToSlash(name)
	hdr := BasicInfoHeader(name, size, fileInfo)

	// If r can be seeked, then this function is two-pass: pass 1 collects the
	// tar header data, and pass 2 copies the data stream. If r cannot be
	// seeked, then some header data (in particular EAs) will be silently lost.
	var (
		restartPos int64
		err        error
	)
	sr, readTwice := r.(io.Seeker)
	if readTwice {
		if restartPos, err = sr.Seek(0, io.SeekCurrent); err != nil {
			readTwice = false
		}
	}

	br := winio.NewBackupStreamReader(r)
	var dataHdr *winio.BackupHeader
	for dataHdr == nil {
		bhdr, err := br.Next()
		if err == io.EOF { //nolint:errorlint
			break
		}
		if err != nil {
			return err
		}
		switch bhdr.Id {
		case winio.BackupData:
			hdr.Mode |= cISREG
			if !readTwice {
				dataHdr = bhdr
			}
		case winio.BackupSecurity:
			sd, err := io.ReadAll(br)
			if err != nil {
				return err
			}
			hdr.PAXRecords[hdrRawSecurityDescriptor] = base64.StdEncoding.EncodeToString(sd)

		case winio.BackupReparseData:
			hdr.Mode |= cISLNK
			hdr.Typeflag = tar.TypeSymlink
			reparseBuffer, _ := io.ReadAll(br)
			rp, err := winio.DecodeReparsePoint(reparseBuffer)
			if err != nil {
				return err
			}
			if rp.IsMountPoint {
				hdr.PAXRecords[hdrMountPoint] = "1"
			}
			hdr.Linkname = rp.Target

		case winio.BackupEaData:
			eab, err := io.ReadAll(br)
			if err != nil {
				return err
			}
			eas, err := winio.DecodeExtendedAttributes(eab)
			if err != nil {
				return err
			}
			for _, ea := range eas {
				// Use base64 encoding for the binary value. Note that there
				// is no way to encode the EA's flags, since their use doesn't
				// make any sense for persisted EAs.
				hdr.PAXRecords[hdrEaPrefix+ea.Name] = base64.StdEncoding.EncodeToString(ea.Value)
			}

		case winio.BackupAlternateData, winio.BackupLink, winio.BackupPropertyData, winio.BackupObjectId, winio.BackupTxfsData:
			// ignore these streams
		default:
			return fmt.Errorf("%s: unknown stream ID %d", name, bhdr.Id)
		}
	}

	err = t.WriteHeader(hdr)
	if err != nil {
		return err
	}

	if readTwice {
		// Get back to the data stream.
		if _, err = sr.Seek(restartPos, io.SeekStart); err != nil {
			return err
		}
		for dataHdr == nil {
			bhdr, err := br.Next()
			if err == io.EOF { //nolint:errorlint
				break
			}
			if err != nil {
				return err
			}
			if bhdr.Id == winio.BackupData {
				dataHdr = bhdr
			}
		}
	}

	// The logic for copying file contents is fairly complicated due to the need for handling sparse files,
	// and the weird ways they are represented by BackupRead. A normal file will always either have a data stream
	// with size and content, or no data stream at all (if empty). However, for a sparse file, the content can also
	// be represented using a series of sparse block streams following the data stream. Additionally, the way sparse
	// files are handled by BackupRead has changed in the OS recently. The specifics of the representation are described
	// in the list at the bottom of this block comment.
	//
	// Sparse files can be represented in four different ways, based on the specifics of the file.
	// - Size = 0:
	//     Previously: BackupRead yields no data stream and no sparse block streams.
	//     Recently: BackupRead yields a data stream with size = 0. There are no following sparse block streams.
	// - Size > 0, no allocated ranges:
	//     BackupRead yields a data stream with size = 0. Following is a single sparse block stream with
	//     size = 0 and offset = <file size>.
	// - Size > 0, one allocated range:
	//     BackupRead yields a data stream with size = <file size> containing the file contents. There are no
	//     sparse block streams. This is the case if you take a normal file with contents and simply set the
	//     sparse flag on it.
	// - Size > 0, multiple allocated ranges:
	//     BackupRead yields a data stream with size = 0. Following are sparse block streams for each allocated
	//     range of the file containing the range contents. Finally there is a sparse block stream with
	//     size = 0 and offset = <file size>.

	if dataHdr != nil { //nolint:nestif // todo: reduce nesting complexity
		// A data stream was found. Copy the data.
		// We assume that we will either have a data stream size > 0 XOR have sparse block streams.
		if dataHdr.Size > 0 || (dataHdr.Attributes&winio.StreamSparseAttributes) == 0 {
			if size != dataHdr.Size {
				return fmt.Errorf("%s: mismatch between file size %d and header size %d", name, size, dataHdr.Size)
			}
			if _, err = io.Copy(t, br); err != nil {
				return fmt.Errorf("%s: copying contents from data stream: %w", name, err)
			}
		} else if size > 0 {
			// As of a recent OS change, BackupRead now returns a data stream for empty sparse files.
			// These files have no sparse block streams, so skip the copySparse call if file size = 0.
			if err = copySparse(t, br); err != nil {
				return fmt.Errorf("%s: copying contents from sparse block stream: %w", name, err)
			}
		}
	}

	// Look for streams after the data stream. The only ones we handle are alternate data streams.
	// Other streams may have metadata that could be serialized, but the tar header has already
	// been written. In practice, this means that we don't get EA or TXF metadata.
	for {
		bhdr, err := br.Next()
		if err == io.EOF { //nolint:errorlint
			break
		}
		if err != nil {
			return err
		}
		switch bhdr.Id {
		case winio.BackupAlternateData:
			if (bhdr.Attributes & winio.StreamSparseAttributes) != 0 {
				// Unsupported for now, since the size of the alternate stream is not present
				// in the backup stream until after the data has been read.
				return fmt.Errorf("%s: tar of sparse alternate data streams is unsupported", name)
			}
			altName := strings.TrimSuffix(bhdr.Name, ":$DATA")
			hdr = &tar.Header{
				Format:     hdr.Format,
				Name:       name + altName,
				Mode:       hdr.Mode,
				Typeflag:   tar.TypeReg,
				Size:       bhdr.Size,
				ModTime:    hdr.ModTime,
				AccessTime: hdr.AccessTime,
				ChangeTime: hdr.ChangeTime,
			}
			err = t.WriteHeader(hdr)
			if err != nil {
				return err
			}
			_, err = io.Copy(t, br)
			if err != nil {
				return err
			}
		case winio.BackupEaData, winio.BackupLink, winio.BackupPropertyData, winio.BackupObjectId, winio.BackupTxfsData:
			// ignore these streams
		default:
			return fmt.Errorf("%s: unknown stream ID %d after data", name, bhdr.Id)
		}
	}
	return nil
}

// FileInfoFromHeader retrieves basic Win32 file information from a tar header, using the additional metadata written by
// WriteTarFileFromBackupStream.
func FileInfoFromHeader(hdr *tar.Header) (name string, size int64, fileInfo *winio.FileBasicInfo, err error) {
	name = hdr.Name
	if hdr.Typeflag == tar.TypeReg || hdr.Typeflag == tar.TypeRegA {
		size = hdr.Size
	}
	fileInfo = &winio.FileBasicInfo{
		LastAccessTime: windows.NsecToFiletime(hdr.AccessTime.UnixNano()),
		LastWriteTime:  windows.NsecToFiletime(hdr.ModTime.UnixNano()),
		ChangeTime:     windows.NsecToFiletime(hdr.ChangeTime.UnixNano()),
		// Default to ModTime, we'll pull hdrCreationTime below if present
		CreationTime: windows.NsecToFiletime(hdr.ModTime.UnixNano()),
	}
	if attrStr, ok := hdr.PAXRecords[hdrFileAttributes]; ok {
		attr, err := strconv.ParseUint(attrStr, 10, 32)
		if err != nil {
			return "", 0, nil, err
		}
		fileInfo.FileAttributes = uint32(attr)
	} else {
		if hdr.Typeflag == tar.TypeDir {
			fileInfo.FileAttributes |= syscall.FILE_ATTRIBUTE_DIRECTORY
		}
	}
	if creationTimeStr, ok := hdr.PAXRecords[hdrCreationTime]; ok {
		creationTime, err := parsePAXTime(creationTimeStr)
		if err != nil {
			return "", 0, nil, err
		}
		fileInfo.CreationTime = windows.NsecToFiletime(creationTime.UnixNano())
	}
	return name, size, fileInfo, err
}

// WriteBackupStreamFromTarFile writes a Win32 backup stream from the current tar file. Since this function may process multiple
// tar file entries in order to collect all the alternate data streams for the file, it returns the next
// tar file that was not processed, or io.EOF is there are no more.
func WriteBackupStreamFromTarFile(w io.Writer, t *tar.Reader, hdr *tar.Header) (*tar.Header, error) {
	bw := winio.NewBackupStreamWriter(w)

	sd, err := SecurityDescriptorFromTarHeader(hdr)
	if err != nil {
		return nil, err
	}
	if len(sd) != 0 {
		bhdr := winio.BackupHeader{
			Id:   winio.BackupSecurity,
			Size: int64(len(sd)),
		}
		err := bw.WriteHeader(&bhdr)
		if err != nil {
			return nil, err
		}
		_, err = bw.Write(sd)
		if err != nil {
			return nil, err
		}
	}

	eadata, err := ExtendedAttributesFromTarHeader(hdr)
	if err != nil {
		return nil, err
	}
	if len(eadata) != 0 {
		bhdr := winio.BackupHeader{
			Id:   winio.BackupEaData,
			Size: int64(len(eadata)),
		}
		err = bw.WriteHeader(&bhdr)
		if err != nil {
			return nil, err
		}
		_, err = bw.Write(eadata)
		if err != nil {
			return nil, err
		}
	}

	if hdr.Typeflag == tar.TypeSymlink {
		reparse := EncodeReparsePointFromTarHeader(hdr)
		bhdr := winio.BackupHeader{
			Id:   winio.BackupReparseData,
			Size: int64(len(reparse)),
		}
		err := bw.WriteHeader(&bhdr)
		if err != nil {
			return nil, err
		}
		_, err = bw.Write(reparse)
		if err != nil {
			return nil, err
		}
	}

	if hdr.Typeflag == tar.TypeReg || hdr.Typeflag == tar.TypeRegA {
		bhdr := winio.BackupHeader{
			Id:   winio.BackupData,
			Size: hdr.Size,
		}
		err := bw.WriteHeader(&bhdr)
		if err != nil {
			return nil, err
		}
		_, err = io.Copy(bw, t)
		if err != nil {
			return nil, err
		}
	}
	// Copy all the alternate data streams and return the next non-ADS header.
	for {
		ahdr, err := t.Next()
		if err != nil {
			return nil, err
		}
		if ahdr.Typeflag != tar.TypeReg || !strings.HasPrefix(ahdr.Name, hdr.Name+":") {
			return ahdr, nil
		}
		bhdr := winio.BackupHeader{
			Id:   winio.BackupAlternateData,
			Size: ahdr.Size,
			Name: ahdr.Name[len(hdr.Name):] + ":$DATA",
		}
		err = bw.WriteHeader(&bhdr)
		if err != nil {
			return nil, err
		}
		_, err = io.Copy(bw, t)
		if err != nil {
			return nil, err
		}
	}
}
