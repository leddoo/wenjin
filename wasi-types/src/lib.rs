use wenjin::*;
use wenjin_derive::CType;
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};




#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, CType)]
#[repr(C)]
pub struct ExitCode(pub u32);



/// A file descriptor handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, CType)]
#[repr(C)]
pub struct Fd(pub u32);


/// Identifier for a device containing a file system. 
/// Can be used in combination with inode to uniquely identify a file or directory in the filesystem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, CType)]
#[repr(C)]
pub struct Device(pub u64);

/// File serial number that is unique within its file system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, CType)]
#[repr(C)]
pub struct Inode(pub u64);


///
#[derive(Clone, Copy, Debug, PartialEq, Eq, CType)]
#[repr(C)]
pub struct FileType(pub u8);

impl FileType {
    /// The type of the file descriptor or file is unknown or is different from any of the other types specified.
    pub const UNKNOWN: FileType = FileType(0);

    /// The file descriptor or file refers to a block device inode.
    pub const BLOCK_DEVICE: FileType = FileType(1);

    /// The file descriptor or file refers to a character device inode.
    pub const CHARACTER_DEVICE: FileType = FileType(2);

    /// The file descriptor or file refers to a directory inode.
    pub const DIRECTORY: FileType = FileType(3);

    /// The file descriptor or file refers to a regular file inode.
    pub const REGULAR_FILE: FileType = FileType(4);

    /// The file descriptor or file refers to a datagram socket.
    pub const SOCKET_DGRAM: FileType = FileType(5);

    /// The file descriptor or file refers to a byte-stream socket.
    pub const SOCKET_STREAM: FileType = FileType(6);

    /// The file refers to a symbolic link inode.
    pub const SYMBOLIC_LINK: FileType = FileType(7);
}


/// Non-negative file size or length of a region within a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, CType)]
#[repr(C)]
pub struct FileSize(pub u64);


/// Relative offset within a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, CType)]
#[repr(C)]
pub struct FileDelta(pub i64);


/// The position relative to which to set the offset of the file descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CType)]
#[repr(C)]
pub struct Whence(pub u8);

impl Whence {
    /// Seek relative to start-of-file.
    pub const SET: Whence = Whence(0);

    /// Seek relative to current position.
    pub const CUR: Whence = Whence(1);

    /// Seek relative to end-of-file.
    pub const END: Whence = Whence(2);
}


/// A reference to the offset of a directory entry.
/// The value 0 signifies the start of the directory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, CType)]
#[repr(C)]
pub struct DirCookie(pub u64);


/// A directory entry.
#[derive(Clone, Copy, Debug, CType)]
#[repr(C)]
pub struct DirEntry {
    /// The offset of the next directory entry stored in this directory.
    next: DirCookie,

    /// The serial number of the file referred to by this directory entry.
    inode: Inode,

    /// The length of the name of the directory entry.
    name_len: u32,

    /// The type of the file referred to by this directory entry.
    ty: FileType,
}


#[derive(Clone, Copy, Debug, CType)]
#[repr(C)]
pub struct PreStatDir {
    pub name_len: WasmSize,
}

impl PreStatDir {
    #[inline(always)]
    pub fn new(data: u32) -> PreStatDir {
        PreStatDir { name_len: WasmSize(data) }
    }
}


#[derive(Clone, Copy, Debug, CType)]
#[repr(C)]
pub struct PreStat {
    pub kind: u8,
    pub data: u32,
}


/// File descriptor rights, determining which actions may be performed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, CType)]
#[repr(C)]
pub struct FdFlags(pub u16);

impl FdFlags {
    /// Append mode: Data written to the file is always appended to the file's end.
    pub const APPEND: FdFlags = FdFlags(0b1);

    /// Write according to synchronized I/O data integrity completion.
    /// Only the data stored in the file is synchronized.
    pub const DSYNC: FdFlags = FdFlags(0b10);

    /// Non-blocking mode.
    pub const NON_BLOCK: FdFlags = FdFlags(0b100);

    /// Synchronized read I/O operations.
    pub const SYNC_READ: FdFlags = FdFlags(0b1000);

    /// Write according to synchronized I/O file integrity completion.
    /// In addition to synchronizing the data stored in the file,
    /// the implementation may also synchronously update the file's metadata.
    pub const SYNC: FdFlags = FdFlags(0b10000);
}


/// File descriptor attributes.
#[derive(Clone, Copy, Debug, CType)]
#[repr(C)]
pub struct FdStat {
    pub filetype: FileType,

    pub flags: FdFlags,

    /// Rights that apply to this file descriptor.
    pub rights_base: Rights,

    /// Maximum set of rights that may be installed on new file descriptors that are created
    /// through this file descriptor, e.g., through path_open.
    pub rights_inheriting: Rights,
}


/// File attributes.
#[derive(Clone, Copy, Debug, CType)]
#[repr(C)]
pub struct FileStat {
    /// Device ID of device containing the file.
    dev: Device, 

    /// File serial number.
    inode: Inode,

    /// File type.
    file_type: FileType,

    /// Number of hard links to the file.
    link_count: u64,

    /// For regular files, the file size in bytes. 
    /// For symbolic links, the length in bytes of the pathname contained in the symbolic link.
    size: FileSize,

    accessed: Timestamp,
    modified: Timestamp,
    changed:  Timestamp,
}


/// Which file time attributes to adjust.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, CType)]
#[repr(C)]
pub struct FstFlags(pub u16);

impl FstFlags {
    /// Adjust the last data access timestamp to the value stored in filestat::atim.
    pub const ACCESSED: FstFlags = FstFlags(0b1); 

    /// Adjust the last data access timestamp to the time of clock clockid::realtime.
    pub const ACCESSED_NOW: FstFlags = FstFlags(0b10); 

    /// Adjust the last data modification timestamp to the value stored in filestat::mtim.
    pub const MODIFIED: FstFlags = FstFlags(0b100); 

    /// Adjust the last data modification timestamp to the time of clock clockid::realtime.
    pub const MODIFIED_NOW: FstFlags = FstFlags(0b1000); 
}


/// Flags determining the method of how paths are resolved.
pub struct LookupFlags(pub u32);

impl LookupFlags {
    /// As long as the resolved path corresponds to a symbolic link, it is expanded.
    pub const FOLLOW_SYMLINKS: LookupFlags = LookupFlags(0b1);
}


/// Open flags used by path_open.
pub struct OpenFlags(pub u16);

impl OpenFlags {
    /// Create file if it does not exist.
    pub const CREATE: OpenFlags = OpenFlags(0b1);

    /// Fail if not a directory.
    pub const DIRECTORY: OpenFlags = OpenFlags(0b10);

    /// Fail if file already exists.
    pub const NOT_EXISTS: OpenFlags = OpenFlags(0b100);

    /// Truncate file to size 0.
    pub const TRUNCATE: OpenFlags = OpenFlags(0b1000);
}



/// Timestamp in nanoseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, CType)]
#[repr(C)]
pub struct Timestamp {
    pub nanos: u64,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, CType)]
#[repr(C)]
pub struct ClockId(pub u32);

impl ClockId {
    /// The clock measuring real time.
    /// Time value zero corresponds with 1970-01-01T00:00:00Z.
    pub const REALTIME: ClockId = ClockId(0);

    /// The store-wide monotonic clock, which is defined as a clock measuring real time,
    /// whose value cannot be adjusted and which cannot have negative clock jumps.
    /// The epoch of this clock is undefined.
    /// The absolute time value of this clock therefore has no meaning.
    pub const MONOTONIC: ClockId = ClockId(1);

    /// The CPU-time clock associated with the current process.
    pub const PROCESS_CPUTIME_ID: ClockId = ClockId(2);

    /// The CPU-time clock associated with the current thread.
    pub const THREAD_CPUTIME_ID: ClockId = ClockId(3);
}



/// Error codes returned by functions.
/// Not all of these error codes are returned by the functions provided by this API;
/// some are used in higher-level library layers,
/// and others are provided merely for alignment with POSIX.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CType)]
#[repr(C)]
pub struct Errno(pub u16);

impl Errno {
    /// No error occurred. System call completed successfully.
    pub const SUCCESS: Errno = Errno(0);

    /// Argument list too long.
    pub const TOO_BIG: Errno = Errno(1);

    /// Permission denied.
    pub const ACCESS: Errno = Errno(2);

    /// Address in use.
    pub const ADDR_IN_USE: Errno = Errno(3);

    /// Address not available.
    pub const ADDR_NOT_AVAILABLE: Errno = Errno(4);

    /// Address family not supported.
    pub const ADDR_FAMILY_NOT_SUPPORTED: Errno = Errno(5);

    /// Resource unavailable, or operation would block.
    pub const AGAIN: Errno = Errno(6);

    /// Connection already in progress.
    pub const ALREADY: Errno = Errno(7);

    /// Bad file descriptor.
    pub const BAD_FD: Errno = Errno(8);

    /// Bad message.
    pub const BAD_MSG: Errno = Errno(9);

    /// Device or resource busy.
    pub const BUSY: Errno = Errno(10);

    /// Operation canceled.
    pub const CANCELED: Errno = Errno(11);

    /// No child processes.
    pub const NO_CHILD: Errno = Errno(12);

    /// Connection aborted.
    pub const CONN_ABORTED: Errno = Errno(13);

    /// Connection refused.
    pub const CONN_REFUSED: Errno = Errno(14);

    /// Connection reset.
    pub const CONN_RESET: Errno = Errno(15);

    /// Resource deadlock would occur.
    pub const DEAD_LOCK: Errno = Errno(16);

    /// Destination address required.
    pub const DEST_ADDR_REQUIRED: Errno = Errno(17);

    /// Mathematics argument out of domain of function.
    pub const DOMAIN: Errno = Errno(18);

    /// Reserved.
    pub const DQUOT: Errno = Errno(19);

    /// File exists.
    pub const FILE_EXISTS: Errno = Errno(20);

    /// Bad address.
    pub const FAULT: Errno = Errno(21);

    /// File too large.
    pub const FILE_TOO_LARGE: Errno = Errno(22);

    /// Host is unreachable.
    pub const HOST_UNREACH: Errno = Errno(23);

    /// Identifier removed.
    pub const IDENT_REMOVED: Errno = Errno(24);

    /// Illegal byte sequence.
    pub const ILLEGAL_SEQ: Errno = Errno(25);

    /// Operation in progress.
    pub const IN_PROGRESS: Errno = Errno(26);

    /// Interrupted function.
    pub const INTERRUPTED: Errno = Errno(27);

    /// Invalid argument.
    pub const INVALID_ARG: Errno = Errno(28);

    /// I/O error.
    pub const IO: Errno = Errno(29);

    /// Socket is connected.
    pub const SOCK_IS_CONNECTED: Errno = Errno(30);

    ///  Is a directory.
    pub const IS_DIR: Errno = Errno(31);

    /// Too many levels of symbolic links.
    pub const LOOP: Errno = Errno(32);

    /// File descriptor value too large.
    pub const FD_TOO_LARGE: Errno = Errno(33);

    /// Too many links.
    pub const TOO_MANY_LINKS: Errno = Errno(34);

    /// Message too large.
    pub const MSG_TOO_LARGE: Errno = Errno(35);

    /// Reserved.
    pub const MULTIHOP: Errno = Errno(36);

    /// Filename too long.
    pub const NAME_TOO_LONG: Errno = Errno(37);

    /// Network is down.
    pub const NET_DOWN: Errno = Errno(38);

    /// Connection aborted by network.
    pub const NET_RESET: Errno = Errno(39);

    /// Network unreachable.
    pub const NET_UNREACH: Errno = Errno(40);

    /// Too many files open in system.
    pub const TOO_MANY_FILES: Errno = Errno(41);

    /// No buffer space available.
    pub const NO_BUFFER_SPACE: Errno = Errno(42);

    /// No such device.
    pub const NO_DEV: Errno = Errno(43);

    /// No such file or directory.
    pub const NO_ENTRY: Errno = Errno(44);

    /// Executable file format error.
    pub const NO_EXEC: Errno = Errno(45);

    /// No locks available.
    pub const NO_LOCK: Errno = Errno(46);

    /// Reserved.
    pub const NO_LINK: Errno = Errno(47);

    /// Not enough space.
    pub const NO_MEM: Errno = Errno(48);

    /// No message of the desired type.
    pub const NO_MSG: Errno = Errno(49);

    ///  Protocol not available.
    pub const NO_PROTO_OPT: Errno = Errno(50);

    /// No space left on device.
    pub const NO_SPACE: Errno = Errno(51);

    /// Function not supported.
    pub const NO_SYS: Errno = Errno(52);

    /// The socket is not connected.
    pub const NOT_CONNECTED: Errno = Errno(53);

    /// Not a directory or a symbolic link to a directory.
    pub const NOT_DIR: Errno = Errno(54);

    /// Directory not empty.
    pub const NOT_EMPTY: Errno = Errno(55);

    /// State not recoverable.
    pub const NOT_RECOVERABLE: Errno = Errno(56);

    /// Not a socket.
    pub const NOT_SOCK: Errno = Errno(57);

    /// Not supported, or operation not supported on socket.
    pub const NOT_SUPPORTED: Errno = Errno(58);

    /// Inappropriate I/O control operation.
    pub const NO_TTY: Errno = Errno(59);

    /// No such device or address.
    pub const NXIO: Errno = Errno(60);

    /// Value too large to be stored in data type.
    pub const OVERFLOW: Errno = Errno(61);

    /// Previous owner died.
    pub const OWNER_DEAD: Errno = Errno(62);

    /// Operation not permitted.
    pub const PERM: Errno = Errno(63);

    /// Broken pipe.
    pub const BROKEN_PIPE: Errno = Errno(64);

    /// Protocol error.
    pub const PROTO_ERR: Errno = Errno(65);

    /// Protocol not supported.
    pub const PROTO_NOT_SUPPORTED: Errno = Errno(66);

    /// Protocol wrong type for socket.
    pub const PROTO_TYPE: Errno = Errno(67);

    /// Result too large.
    pub const RANGE: Errno = Errno(68);

    /// Read-only file system.
    pub const READ_ONLY_FS: Errno = Errno(69);

    /// Invalid seek.
    pub const INVALID_SEEK: Errno = Errno(70);

    /// No such process.
    pub const NO_PROCESS: Errno = Errno(71);

    /// Reserved.
    pub const STALE: Errno = Errno(72);

    /// Connection timed out.
    pub const TIMED_OUT: Errno = Errno(73);

    /// Text file busy.
    pub const TXT_BUSY: Errno = Errno(74);

    /// Cross-device link.
    pub const XDEV: Errno = Errno(75);

    /// Extension: Capabilities insufficient.
    pub const NOT_CAPABLE: Errno = Errno(76);
}



/// File descriptor rights, determining which actions may be performed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, CType)]
#[repr(C)]
pub struct Rights(pub u64);

impl Rights {
    /// The right to invoke fd_datasync.
    /// If path_open is set, includes the right to invoke path_open with fdflags::dsync.
    pub const FD_DATASYNC: Rights = Rights(0b1);

    /// The right to invoke fd_read and sock_recv.
    /// If rights::fd_seek is set, includes the right to invoke fd_pread.
    pub const FD_READ: Rights = Rights(0b10);

    /// The right to invoke fd_seek. This flag implies rights::fd_tell.
    pub const FD_SEEK: Rights = Rights(0b100);

    /// The right to invoke fd_fdstat_set_flags.
    pub const FD_FDSTAT_SET_FLAGS: Rights = Rights(0b1000);

    /// The right to invoke fd_sync.
    /// If path_open is set, includes the right to invoke path_open with fdflags::rsync and fdflags::dsync.
    pub const FD_SYNC: Rights = Rights(0b10000);

    /// The right to invoke fd_seek in such a way that the file offset remains unaltered
    /// (i.e., whence::cur with offset zero), or to invoke fd_tell.
    pub const FD_TELL: Rights = Rights(0b100000);

    /// The right to invoke fd_write and sock_send.
    /// If rights::fd_seek is set, includes the right to invoke fd_pwrite.
    pub const FD_WRITE: Rights = Rights(0b1000000);

    /// The right to invoke fd_advise.
    pub const FD_ADVISE: Rights = Rights(0b10000000);

    /// The right to invoke fd_allocate.
    pub const FD_ALLOCATE: Rights = Rights(0b100000000);

    /// The right to invoke path_create_directory.
    pub const PATH_CREATE_DIRECTORY: Rights = Rights(0b1000000000);

    /// If path_open is set, the right to invoke path_open with oflags::creat.
    pub const PATH_CREATE_FILE: Rights = Rights(0b10000000000);

    /// The right to invoke path_link with the file descriptor as the source directory.
    pub const PATH_LINK_SOURCE: Rights = Rights(0b100000000000);

    /// The right to invoke path_link with the file descriptor as the target directory.
    pub const PATH_LINK_TARGET: Rights = Rights(0b1000000000000);

    /// The right to invoke path_open.
    pub const PATH_OPEN: Rights = Rights(0b10000000000000);

    /// The right to invoke fd_readdir.
    pub const FD_READDIR: Rights = Rights(0b100000000000000);

    /// The right to invoke path_readlink.
    pub const PATH_READLINK: Rights = Rights(0b1000000000000000);

    /// The right to invoke path_rename with the file descriptor as the source directory.
    pub const PATH_RENAME_SOURCE: Rights = Rights(0b10000000000000000);

    /// The right to invoke path_rename with the file descriptor as the target directory.
    pub const PATH_RENAME_TARGET: Rights = Rights(0b100000000000000000);

    /// The right to invoke path_filestat_get.
    pub const PATH_FILESTAT_GET: Rights = Rights(0b1000000000000000000);

    /// The right to change a file's size (there is no path_filestat_set_size).
    /// If path_open is set, includes the right to invoke path_open with oflags::trunc.
    pub const PATH_FILESTAT_SET_SIZE: Rights = Rights(0b10000000000000000000);

    /// The right to invoke path_filestat_set_times.
    pub const PATH_FILESTAT_SET_TIMES: Rights = Rights(0b100000000000000000000);

    /// The right to invoke fd_filestat_get.
    pub const FD_FILESTAT_GET: Rights = Rights(0b1000000000000000000000);

    /// The right to invoke fd_filestat_set_size.
    pub const FD_FILESTAT_SET_SIZE: Rights = Rights(0b10000000000000000000000);

    /// The right to invoke fd_filestat_set_times.
    pub const FD_FILESTAT_SET_TIMES: Rights = Rights(0b100000000000000000000000);

    /// The right to invoke path_symlink.
    pub const PATH_SYMLINK: Rights = Rights(0b1000000000000000000000000);

    /// The right to invoke path_remove_directory.
    pub const PATH_REMOVE_DIRECTORY: Rights = Rights(0b10000000000000000000000000);

    /// The right to invoke path_unlink_file.
    pub const PATH_UNLINK_FILE: Rights = Rights(0b100000000000000000000000000);

    /// If rights::fd_read is set, includes the right to invoke poll_oneoff to subscribe to eventtype::fd_read.
    /// If rights::fd_write is set, includes the right to invoke poll_oneoff to subscribe to eventtype::fd_write.
    pub const POLL_FD_READWRITE: Rights = Rights(0b1000000000000000000000000000);

    /// The right to invoke sock_shutdown.
    pub const SOCK_SHUTDOWN: Rights = Rights(0b10000000000000000000000000000);
}



/// File or memory access pattern advisory information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CType)]
#[repr(C)]
pub struct Advice(pub u8);

impl Advice {
    /// The application has no advice to give on its behavior with respect to the specified data.
    pub const NORMAL: Advice = Advice(0);

    /// The application expects to access the specified data sequentially from lower offsets to higher offsets.
    pub const SEQUENTIAL: Advice = Advice(1);

    /// The application expects to access the specified data in a random order.
    pub const RANDOM: Advice = Advice(2);

    /// The application expects to access the specified data in the near future.
    pub const WILL_NEED: Advice = Advice(3);

    /// The application expects that it will not access the specified data in the near future.
    pub const DONT_NEED: Advice = Advice(4);

    /// The application expects to access the specified data once and then not reuse it thereafter.
    pub const NO_REUSE: Advice = Advice(5);
}



// @TODO.
/// Subscription to an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CType)]
#[repr(C)]
pub struct Subscription;

// @TODO.
/// An event that occurred.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CType)]
#[repr(C)]
pub struct Event;



/// Signal condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CType)]
#[repr(C)]
pub struct Signal(pub u8);

impl Signal {
    /// No signal. Note that POSIX has special semantics for kill(pid, 0), so this value is reserved.
    pub const NONE: Signal = Signal(0);

    /// Hangup. Action: Terminates the process.
    pub const HUP: Signal = Signal(1);

    /// Terminate interrupt signal. Action: Terminates the process.
    pub const INT: Signal = Signal(2);

    /// Terminal quit signal. Action: Terminates the process.
    pub const QUIT: Signal = Signal(3);

    /// Illegal instruction. Action: Terminates the process.
    pub const ILL: Signal = Signal(4);

    /// Trace/breakpoint trap. Action: Terminates the process.
    pub const TRAP: Signal = Signal(5);

    /// Process abort signal. Action: Terminates the process.
    pub const ABRT: Signal = Signal(6);

    /// Access to an undefined portion of a memory object. Action: Terminates the process.
    pub const BUS: Signal = Signal(7);

    /// Erroneous arithmetic operation. Action: Terminates the process.
    pub const FPE: Signal = Signal(8);

    /// Kill. Action: Terminates the process.
    pub const KILL: Signal = Signal(9);

    /// User-defined signal 1. Action: Terminates the process.
    pub const USR1: Signal = Signal(10);

    /// Invalid memory reference. Action: Terminates the process.
    pub const SEGV: Signal = Signal(11);

    /// User-defined signal 2. Action: Terminates the process.
    pub const USR2: Signal = Signal(12);

    /// Write on a pipe with no one to read it. Action: Ignored.
    pub const PIPE: Signal = Signal(13);

    /// Alarm clock. Action: Terminates the process.
    pub const ALRM: Signal = Signal(14);

    /// Termination signal. Action: Terminates the process.
    pub const TERM: Signal = Signal(15);

    /// Child process terminated, stopped, or continued. Action: Ignored.
    pub const CHLD: Signal = Signal(16);

    /// Continue executing, if stopped. Action: Continues executing, if stopped.
    pub const CONT: Signal = Signal(17);

    /// Stop executing. Action: Stops executing.
    pub const STOP: Signal = Signal(18);

    /// Terminal stop signal. Action: Stops executing.
    pub const TSTP: Signal = Signal(19);

    /// Background process attempting read. Action: Stops executing.
    pub const TTIN: Signal = Signal(20);

    /// Background process attempting write. Action: Stops executing.
    pub const TTOU: Signal = Signal(21);

    /// High bandwidth data is available at a socket. Action: Ignored.
    pub const URG: Signal = Signal(22);

    /// CPU time limit exceeded. Action: Terminates the process.
    pub const XCPU: Signal = Signal(23);

    /// File size limit exceeded. Action: Terminates the process.
    pub const XFSZ: Signal = Signal(24);

    /// Virtual timer expired. Action: Terminates the process.
    pub const VTALRM: Signal = Signal(25);

    /// Profiling timer expired. Action: Terminates the process.
    pub const PROF: Signal = Signal(26);

    /// Window changed. Action: Ignored.
    pub const WINCH: Signal = Signal(27);

    /// I/O possible. Action: Terminates the process.
    pub const POLL: Signal = Signal(28);

    /// Power failure. Action: Terminates the process.
    pub const PWR: Signal = Signal(29);

    /// Bad system call. Action: Terminates the process.
    pub const SYS: Signal = Signal(30);
}



/// Flags provided to sock_recv.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, CType)]
#[repr(C)]
pub struct SockRecvInFlags(pub u16);

impl SockRecvInFlags {
    /// Returns the message without removing it from the socket's receive queue.
    pub const PEEK: SockRecvInFlags = SockRecvInFlags(0b1);

    /// On byte-stream sockets, block until the full amount of data can be returned.
    pub const WAIT_ALL: SockRecvInFlags = SockRecvInFlags(0b10);
}


/// Flags returned by sock_recv.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, CType)]
#[repr(C)]
pub struct SockRecvOutFlags(pub u16);

impl SockRecvOutFlags {
    /// Message data has been truncated.
    pub const DATA_TRUNCATED: SockRecvOutFlags= SockRecvOutFlags(0b1);
}


/// Flags provided to sock_send. 
/// As there are currently no flags defined, it must be set to zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, CType)]
#[repr(C)]
pub struct SockSendFlags(pub u16);


/// Which channels on a socket to shut down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, CType)]
#[repr(C)]
pub struct SockShutdownFlags(pub u8);

impl SockShutdownFlags {
    /// Disables further receive operations.
    pub const RECV: SockShutdownFlags = SockShutdownFlags(0b1);

    /// Disables further send operations.
    pub const SEND: SockShutdownFlags = SockShutdownFlags(0b10);
}



/// A region of memory for scatter/gather reads.
#[derive(Clone, Copy, Debug, CType)]
#[repr(C)]
pub struct IoVec {
    ///  The address of the buffer to be filled.
    pub buf: WasmPtr<u8>,
    ///  The length of the buffer to be filled.
    pub len: WasmSize,
}


/// A (constant) region of memory for scatter/gather reads.
pub type CIoVec = IoVec;



#[derive(Default)]
pub struct Funcs<T: Clone + 'static> {
    /// Return command-line argument data sizes.
    /// results:
    ///  - argc: The number of arguments.
    ///  - argv_buf_size: The size of the argument string data.
    pub args_sizes_get: Option<fn(&T, &mut MemoryView) -> Result<(Errno, WasmSize, WasmSize), ()>>,

    /// Read command-line argument data.
    /// The size of the array should match that returned by args_sizes_get.
    pub args_get: Option<fn(&T, &mut MemoryView, argv: WasmPtr<WasmPtr<u8>>, buf: WasmPtr<u8>) -> Result<Errno, ()>>,


    /// Return environment variable data sizes.
    /// results:
    ///  - environc: The number of environment variable arguments.
    ///  - environ_buf_size: The size of the environment variable data.
    pub environ_sizes_get: Option<fn(&T, &mut MemoryView) -> Result<(Errno, WasmSize, WasmSize), ()>>,

    /// Read environment variable data. The sizes of the buffers should match that returned by environ_sizes_get.
    pub environ_get: Option<fn(&T, &mut MemoryView, environ: WasmPtr<WasmPtr<u8>>, environ_buf: WasmPtr<u8>) -> Result<Errno, ()>>,


    /// Return the resolution of a clock.
    /// Implementations are required to provide a non-zero value for supported clocks.
    /// For unsupported clocks, return errno::inval.
    /// Note: This is similar to clock_getres in POSIX.
    pub clock_res_get: Option<fn(&T, &mut MemoryView, id: ClockId) -> Result<(Errno, Timestamp), ()>>,

    /// Return the time value of a clock. Note: This is similar to clock_gettime in POSIX.
    /// params:
    ///  - precision: timestamp The maximum lag (exclusive) that the returned time value may have,
    ///    compared to its actual value.
    pub clock_time_get: Option<fn(&T, &mut MemoryView, id: ClockId, precision: Timestamp) -> Result<(Errno, Timestamp), ()>>,


    /// Provide file advisory information on a file descriptor.
    /// Note: This is similar to posix_fadvise in POSIX.
    pub fd_advise: Option<fn(&T, &mut MemoryView, Fd, offset: FileSize, len: FileSize, advice: Advice) -> Result<Errno, ()>>,

    /// Force the allocation of space in a file.
    /// Note: This is similar to posix_fallocate in POSIX.
    pub fd_allocate: Option<fn(&T, &mut MemoryView, Fd, offset: FileSize, len: FileSize) -> Result<Errno, ()>>,

    /// Close a file descriptor. Note: This is similar to close in POSIX.
    pub fd_close: Option<fn(&T, &mut MemoryView, Fd) -> Result<Errno, ()>>,

    /// Synchronize the data of a file to disk.
    /// Note: This is similar to fdatasync in POSIX.
    pub fd_datasync: Option<fn (&T, &mut MemoryView, Fd) -> Result<Errno, ()>>,

    /// Get the attributes of a file descriptor.
    /// Note: This returns similar flags to fsync(fd, F_GETFL) in POSIX, as well as additional fields.
    pub fd_fdstat_get: Option<fn(&T, &mut MemoryView, Fd) -> Result<(Errno, FdStat), ()>>,

    /// Adjust the flags associated with a file descriptor.
    /// Note: This is similar to fcntl(fd, F_SETFL, flags) in POSIX.
    pub fd_fdstat_set_flags: Option<fn(&T, &mut MemoryView, Fd, flags: FdFlags) -> Result<Errno, ()>>,

    /// Adjust the rights associated with a file descriptor.
    /// This can only be used to remove rights, and returns errno::notcapable
    /// if called in a way that would attempt to add rights.
    pub fd_fdstat_set_rights: Option<fn(&T, &mut MemoryView, Fd, rights_base: Rights, rights_inheriting: Rights) -> Result<Errno, ()>>,

    /// Return the attributes of an open file.
    pub fd_filestat_get: Option<fn(&T, &mut MemoryView, Fd) -> Result<(Errno, FileStat), ()>>,

    /// Adjust the size of an open file.
    /// If this increases the file's size, the extra bytes are filled with zeros.
    /// Note: This is similar to ftruncate in POSIX.
    pub fd_filestat_set_size: Option<fn(&T, &mut MemoryView, Fd, size: FileSize) -> Result<Errno, ()>>,

    /// Adjust the timestamps of an open file or directory.
    /// Note: This is similar to futimens in POSIX.
    /// params:
    ///  - fst_flags: A bitmask indicating which timestamps to adjust.
    pub fd_filestat_set_times: Option<fn(&T, &mut MemoryView, Fd, accessed: Timestamp, modified: Timestamp, fst_flags: FstFlags) -> Result<Errno, ()>>,

    /// Read from a file descriptor, without using and updating the file descriptor's offset.
    /// Note: This is similar to preadv in POSIX.
    /// params:
    ///  - offset: The offset within the file at which to read.
    /// results:
    ///  - nread: The number of bytes read.
    pub fd_pread: Option<fn(&T, &mut MemoryView, Fd, iovs: WasmPtr<CIoVec>, iovs_len: WasmSize, offset: FileSize) -> Result<(Errno, WasmSize), ()>>,

    /// Return a description of the given preopened file descriptor.
    pub fd_prestat_get: Option<fn(&T, &mut MemoryView, Fd) -> Result<(Errno, PreStat), ()>>,

    /// Return a description of the given preopened file descriptor.
    /// params:
    ///  - path: A buffer into which to write the preopened directory name.
    pub fd_prestat_dir_name: Option<fn(&T, &mut MemoryView, Fd, path: WasmPtr<u8>, path_len: WasmSize) -> Result<Errno, ()>>,

    /// Write to a file descriptor, without using and updating the file descriptor's offset.
    /// Note: This is similar to pwritev in POSIX.
    /// params:
    ///  - offset: The offset within the file at which to write.
    /// results:
    ///  - nwritten: The number of bytes written.
    pub fd_pwrite: Option<fn(&T, &mut MemoryView, Fd, iovs: WasmPtr<IoVec>, iovs_len: WasmSize, offset: FileSize) -> Result<(Errno, WasmSize), ()>>,

    /// Read from a file descriptor. Note: This is similar to readv in POSIX.
    /// results:
    ///  - nread: The number of bytes read.
    pub fd_read: Option<fn(&T, &mut MemoryView, Fd, iovs: WasmPtr<CIoVec>, iovs_len: WasmSize) -> Result<(Errno, WasmSize), ()>>,

    /// Read directory entries from a directory.
    /// When successful, the contents of the output buffer consist of a sequence of directory entries.
    /// Each directory entry consists of a dirent object, followed by dirent::d_namlen bytes
    /// holding the name of the directory entry.
    /// This function fills the output buffer as much as possible,
    /// potentially truncating the last directory entry.
    /// This allows the caller to grow its read buffer size in case it's too small to fit
    /// a single large directory entry, or skip the oversized directory entry.
    /// results:
    ///  - bufused: The number of bytes stored in the read buffer.
    ///    If less than the size of the read buffer, the end of the directory has been reached.
    pub fd_readdir: Option<fn(&T, &mut MemoryView, Fd, buf: WasmPtr<u8>, buf_len: WasmSize, cookie: DirCookie) -> Result<(Errno, WasmSize), ()>>,

    /// Atomically replace a file descriptor by renumbering another file descriptor.
    /// Due to the strong focus on thread safety, this environment does not provide a mechanism to
    /// duplicate or renumber a file descriptor to an arbitrary number, like dup2().
    /// This would be prone to race conditions, as an actual file descriptor with the same number
    /// could be allocated by a different thread at the same time.
    /// This function provides a way to atomically renumber file descriptors,
    /// which would disappear if dup2() were to be removed entirely.
    pub fd_renumber: Option<fn(&T, &mut MemoryView, from: Fd, to: Fd) -> Result<Errno, ()>>,

    /// Move the offset of a file descriptor.
    /// Note: This is similar to lseek in POSIX.
    /// params:
    ///  - delta: The number of bytes to move.
    ///  - whence: The base from which the offset is relative.
    /// results:
    ///  - newoffset: filesize The new offset of the file descriptor, relative to the start of the file.
    pub fd_seek: Option<fn(&T, &mut MemoryView, Fd, delta: FileDelta, whence: Whence) -> Result<(Errno, FileSize), ()>>,

    /// Synchronize the data and metadata of a file to disk.
    /// Note: This is similar to fsync in POSIX.
    pub fd_sync: Option<fn(&T, &mut MemoryView, Fd) -> Result<Errno, ()>>,

    /// Return the current offset of a file descriptor.
    /// Note: This is similar to lseek(fd, 0, SEEK_CUR) in POSIX.
    /// results:
    ///  - offset: The current offset of the file descriptor, relative to the start of the file.
    pub fd_tell: Option<fn(&T, &mut MemoryView, Fd) -> Result<(Errno, FileSize), ()>>,

    /// Write to a file descriptor. Note: This is similar to writev in POSIX.
    /// results:
    ///  - nwritten: The number of bytes written.
    pub fd_write: Option<fn(&T, &mut MemoryView, Fd, iovs: WasmPtr<IoVec>, iovs_len: WasmSize) -> Result<(Errno, WasmSize), ()>>,


    /// Create a directory.
    /// Note: This is similar to mkdirat in POSIX.
    pub path_create_directory: Option<fn(&T, &mut MemoryView, Fd, path: WasmPtr<u8>, path_len: WasmSize) -> Result<Errno, ()>>,

    /// Return the attributes of a file or directory.
    /// Note: This is similar to stat in POSIX.
    pub path_filestat_get: Option<fn(&T, &mut MemoryView, Fd, flags: LookupFlags, path: WasmPtr<u8>, path_len: WasmSize) -> Result<(Errno, FileStat), ()>>,

    /// Adjust the timestamps of a file or directory.
    /// Note: This is similar to utimensat in POSIX.
    /// params:
    ///  - fst_flags: A bitmask indicating which timestamps to adjust.
    pub path_filestat_set_times: Option<fn(&T, &mut MemoryView, Fd, flags: LookupFlags, path: WasmPtr<u8>, path_len: WasmSize, accessed: Timestamp, modified: Timestamp, fst_flags: FstFlags) -> Result<Errno, ()>>,

    /// Create a hard link.
    /// Note: This is similar to linkat in POSIX.
    pub path_link: Option<fn(&T, &mut MemoryView, old_fd: Fd, old_flags: LookupFlags, old_path: WasmPtr<u8>, old_path_len: WasmSize, new_fd: Fd, new_path: WasmPtr<u8>, new_path_len: WasmSize) -> Result<Errno, ()>>,

    /// Open a file or directory.
    /// The returned file descriptor is not guaranteed to be the lowest-numbered file descriptor
    /// not currently open; it is randomized to prevent applications from depending on making assumptions
    /// about indexes, since this is error-prone in multi-threaded contexts.
    /// The returned file descriptor is guaranteed to be less than 2**31.
    /// Note: This is similar to openat in POSIX.
    /// params:
    ///  - fs_rights_base/inherited: The initial rights of the newly created file descriptor.
    ///    The implementation is allowed to return a file descriptor with fewer rights than specified,
    ///    if and only if those rights do not apply to the type of file being opened.
    ///    The base rights are rights that will apply to operations using the file descriptor itself,
    ///    while the inheriting rights are rights that apply to file descriptors derived from it.
    pub path_open: Option<fn(&T, &mut MemoryView, Fd, flags: LookupFlags, path: WasmPtr<u8>, path_len: WasmSize, open_flags: OpenFlags, rights_base: Rights, rights_inheriting: Rights, fd_flags: FdFlags) -> Result<(Errno, Fd), ()>>,

    /// Read the contents of a symbolic link.
    /// Note: This is similar to readlinkat in POSIX.
    /// params:
    ///  - path: The path of the symbolic link from which to read.
    ///  - buf: The buffer to which to write the contents of the symbolic link.
    /// results:
    ///  - buf_used: The number of bytes placed in the buffer.
    pub path_readlink: Option<fn(&T, &mut MemoryView, Fd, path: WasmPtr<u8>, path_len: WasmSize, buf: WasmPtr<u8>, buf_len: WasmSize) -> Result<(Errno, WasmSize), ()>>,

    /// Remove a directory.
    /// Return errno::notempty if the directory is not empty.
    /// Note: This is similar to unlinkat(fd, path, AT_REMOVEDIR) in POSIX.
    pub path_remove_directory: Option<fn(&T, &mut MemoryView, Fd, path: WasmPtr<u8>, path_len: WasmSize) -> Result<Errno, ()>>,

    /// Rename a file or directory.
    /// Note: This is similar to renameat in POSIX.
    pub path_rename: Option<fn(&T, &mut MemoryView, old_base: Fd, old_path: WasmPtr<u8>, old_path_len: WasmSize, new_base: Fd, new_path: WasmPtr<u8>, new_path_len: WasmSize) -> Result<Errno, ()>>,

    /// Create a symbolic link.
    /// Note: This is similar to symlinkat in POSIX.
    /// params:
    ///  - old_path: The contents of the symbolic link.
    ///  - new_path: The destination path at which to create the symbolic link.
    pub path_symlink: Option<fn(&T, &mut MemoryView, old_path: WasmPtr<u8>, old_path_len: WasmSize, base: Fd, new_path: WasmPtr<u8>, new_path_len: WasmSize) -> Result<Errno, ()>>,

    /// Unlink a file.
    /// Return errno::isdir if the path refers to a directory.
    /// Note: This is similar to unlinkat(fd, path, 0) in POSIX.
    pub path_unlink_file: Option<fn(&T, &mut MemoryView, base: Fd, path: WasmPtr<u8>, WasmSize) -> Result<Errno, ()>>,


    /// Concurrently poll for the occurrence of a set of events.
    /// params:
    ///  - subs: The events to which to subscribe.
    ///  - out: The events that have occurred.
    ///  - num_subs: Both the number of subscriptions and events.
    /// results:
    ///  - num_events: The number of events stored.
    pub poll_oneoff: Option<fn(&T, &mut MemoryView, subs: WasmPtr<Subscription>, out: WasmPtr<Event>, num_subs: WasmSize) -> Result<(Errno, WasmSize), ()>>,


    /// Terminate the process normally.
    /// An exit code of 0 indicates successful termination of the program.
    /// The meanings of other values is dependent on the environment.
    pub proc_exit: Option<fn(&T, &mut MemoryView, exit_code: ExitCode) -> Result<(), ()>>,

    /// Send a signal to the process of the calling thread.
    /// Note: This is similar to raise in POSIX.
    pub proc_raise: Option<fn(&T, &mut MemoryView, Signal) -> Result<Errno, ()>>,


    /// Temporarily yield execution of the calling thread.
    /// Note: This is similar to sched_yield in POSIX.
    pub sched_yield: Option<fn(&T, &mut MemoryView) -> Result<Errno, ()>>,


    /// Write high-quality random data into a buffer.
    /// This function blocks when the implementation is unable to immediately provide
    /// sufficient high-quality random data.
    /// This function may execute slowly, so when large mounts of random data are required,
    /// it's advisable to use this function to seed a pseudo-random number generator,
    /// rather than to provide the random data directly.
    pub random_get: Option<fn(&T, &mut MemoryView, buf: WasmPtr<u8>, buf_len: WasmSize) -> Result<Errno, ()>>,


    /// Accept a new incoming connection.
    /// Note: This is similar to `accept` in POSIX.
    pub sock_accept: Option<fn(&T, &mut MemoryView, Fd, flags: FdFlags) -> Result<(Errno, Fd), ()>>,

    /// Receive a message from a socket. 
    /// Note: This is similar to recv in POSIX, though it also supports reading the data into 
    /// multiple buffers in the manner of readv.
    pub sock_recv: Option<fn(&T, &mut MemoryView, Fd, iovs: WasmPtr<IoVec>, iovs_len: WasmSize, flags: SockRecvInFlags) -> Result<(Errno, WasmSize, SockRecvOutFlags), ()>>,

    /// Send a message on a socket. 
    /// Note: This is similar to send in POSIX, though it also supports writing the data from 
    /// multiple buffers in the manner of writev.
    pub sock_send: Option<fn(&T, &mut MemoryView, Fd, iovs: WasmPtr<CIoVec>, iovs_len: WasmSize, flags: SockSendFlags) -> Result<(Errno, WasmSize), ()>>,

    /// Shut down socket send and receive channels. 
    /// Note: This is similar to shutdown in POSIX.
    pub sock_shutdown: Option<fn(&T, &mut MemoryView, Fd, flags: SockShutdownFlags) -> Result<Errno, ()>>,
}

impl<T: Clone> Funcs<T> {
    pub fn unimplemented() -> Funcs<T> {
        Funcs {
            args_sizes_get: Some(|_, _| { unimplemented!() }),
            args_get: Some(|_, _, _, _| { unimplemented!() }),

            environ_sizes_get: Some(|_, _| { unimplemented!() }),
            environ_get: Some(|_, _, _, _| { unimplemented!() }),

            clock_res_get: Some(|_, _, _| { unimplemented!() }),
            clock_time_get: Some(|_, _, _, _| { unimplemented!() }),

            fd_advise: Some(|_, _, _, _, _, _| { unimplemented!() }),
            fd_allocate: Some(|_, _, _, _, _| { unimplemented!() }),
            fd_close: Some(|_, _, _| { unimplemented!() }),
            fd_datasync: Some(|_, _, _| { unimplemented!() }),
            fd_fdstat_get: Some(|_, _, _| { unimplemented!() }),
            fd_fdstat_set_flags: Some(|_, _, _, _| { unimplemented!() }),
            fd_fdstat_set_rights: Some(|_, _, _, _, _| { unimplemented!() }),
            fd_filestat_get: Some(|_, _, _| { unimplemented!() }),
            fd_filestat_set_size: Some(|_, _, _, _| { unimplemented!() }),
            fd_filestat_set_times: Some(|_, _, _, _, _, _| { unimplemented!() }),
            fd_pread: Some(|_, _, _, _, _, _| { unimplemented!() }),
            fd_prestat_get: Some(|_, _, _| { unimplemented!() }),
            fd_prestat_dir_name: Some(|_, _, _, _, _| { unimplemented!() }),
            fd_pwrite: Some(|_, _, _, _, _, _| { unimplemented!() }),
            fd_read: Some(|_, _, _, _, _| { unimplemented!() }),
            fd_readdir: Some(|_, _, _, _, _, _| { unimplemented!() }),
            fd_renumber: Some(|_, _, _, _| { unimplemented!() }),
            fd_seek: Some(|_, _, _, _, _| { unimplemented!() }),
            fd_sync: Some(|_, _, _| { unimplemented!() }),
            fd_tell: Some(|_, _, _| { unimplemented!() }),
            fd_write: Some(|_, _, _, _, _| { unimplemented!() }),

            path_create_directory: Some(|_, _, _, _, _| { unimplemented!() }),
            path_filestat_get: Some(|_, _, _, _, _, _| { unimplemented!() }),
            path_filestat_set_times: Some(|_, _, _, _, _, _, _, _, _| { unimplemented!() }),
            path_link: Some(|_, _, _, _, _, _, _, _, _| { unimplemented!() }),
            path_open: Some(|_, _, _, _, _, _, _, _, _, _| { unimplemented!() }),
            path_readlink: Some(|_, _, _, _, _, _, _| { unimplemented!() }),
            path_remove_directory: Some(|_, _, _, _, _| { unimplemented!() }),
            path_rename: Some(|_, _, _, _, _, _, _, _| { unimplemented!() }),
            path_symlink: Some(|_, _, _, _, _, _, _| { unimplemented!() }),
            path_unlink_file: Some(|_, _, _, _, _| { unimplemented!() }),

            poll_oneoff: Some(|_, _, _, _, _| { unimplemented!() }),

            proc_exit: Some(|_, _, _| { unimplemented!() }),
            proc_raise: Some(|_, _, _| { unimplemented!() }),

            sched_yield: Some(|_, _| { unimplemented!() }),

            random_get: Some(|_, _, _, _| { unimplemented!() }),

            sock_accept: Some(|_, _, _, _| { unimplemented!() }),
            sock_recv: Some(|_, _, _, _, _, _| { unimplemented!() }),
            sock_send: Some(|_, _, _, _, _, _| { unimplemented!() }),
            sock_shutdown: Some(|_, _, _, _| { unimplemented!() }),
        }
    }
}


#[derive(Default)]
pub struct StoreFuncs {
    pub args_sizes_get: Option<Func>,
    pub args_get: Option<Func>,

    pub environ_sizes_get: Option<Func>,
    pub environ_get: Option<Func>,

    pub clock_res_get: Option<Func>,
    pub clock_time_get: Option<Func>,

    pub fd_advise: Option<Func>,
    pub fd_allocate: Option<Func>,
    pub fd_close: Option<Func>,
    pub fd_datasync: Option<Func>,
    pub fd_fdstat_get: Option<Func>,
    pub fd_fdstat_set_flags: Option<Func>,
    pub fd_fdstat_set_rights: Option<Func>,
    pub fd_filestat_get: Option<Func>,
    pub fd_filestat_set_size: Option<Func>,
    pub fd_filestat_set_times: Option<Func>,
    pub fd_pread: Option<Func>,
    pub fd_prestat_get: Option<Func>,
    pub fd_prestat_dir_name: Option<Func>,
    pub fd_pwrite: Option<Func>,
    pub fd_read: Option<Func>,
    pub fd_readdir: Option<Func>,
    pub fd_renumber: Option<Func>,
    pub fd_seek: Option<Func>,
    pub fd_sync: Option<Func>,
    pub fd_tell: Option<Func>,
    pub fd_write: Option<Func>,

    pub path_create_directory: Option<Func>,
    pub path_filestat_get: Option<Func>,
    pub path_filestat_set_times: Option<Func>,
    pub path_link: Option<Func>,
    pub path_open: Option<Func>,
    pub path_readlink: Option<Func>,
    pub path_remove_directory: Option<Func>,
    pub path_rename: Option<Func>,
    pub path_symlink: Option<Func>,
    pub path_unlink_file: Option<Func>,

    pub poll_oneoff: Option<Func>,

    pub proc_exit: Option<Func>,
    pub proc_raise: Option<Func>,

    pub sched_yield: Option<Func>,

    pub random_get: Option<Func>,

    pub sock_accept: Option<Func>,
    pub sock_recv: Option<Func>,
    pub sock_send: Option<Func>,
    pub sock_shutdown: Option<Func>,
}

impl<T: Clone + 'static> Funcs<T> {
    pub fn register(&self, this: T, store: &mut Store) -> StoreFuncs {
        let mut result = StoreFuncs::default();

        if let Some(args_sizes_get) = self.args_sizes_get {
            let this = this.clone();
            result.args_sizes_get = Some(store.add_func(move |mem: &mut MemoryView, argc_ptr: WasmPtr<WasmSize>, argv_buf_size_ptr: WasmPtr<WasmSize>| {
                let (errno, argc, argv_buf_size) = args_sizes_get(&this, mem)?;
                mem.write(argc_ptr, argc)?;
                mem.write(argv_buf_size_ptr, argv_buf_size)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(args_get) = self.args_get {
            let this = this.clone();
            result.args_get = Some(store.add_func(move |mem: &mut MemoryView, argv, buf| {
                return Ok(args_get(&this, mem, argv, buf)?.0 as u32);
            }).func());
        }

        if let Some(environ_sizes_get) = self.environ_sizes_get {
            let this = this.clone();
            result.environ_sizes_get = Some(store.add_func(move |mem: &mut MemoryView, environc_ptr: WasmPtr<WasmSize>, environ_buf_size_ptr: WasmPtr<WasmSize>| {
                let (errno, environc, environ_buf_size) = environ_sizes_get(&this, mem)?;
                mem.write(environc_ptr, environc)?;
                mem.write(environ_buf_size_ptr, environ_buf_size)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(environ_get) = self.environ_get {
            let this = this.clone();
            result.environ_get = Some(store.add_func(move |mem: &mut MemoryView, environ, environ_buf| {
                let errno = environ_get(&this, mem, environ, environ_buf)?;
                return Ok(errno.0 as u32);
            }).func());
        }

        if let Some(clock_res_get) = self.clock_res_get {
            let this = this.clone();
            result.clock_res_get = Some(store.add_func(move |mem: &mut MemoryView, clock: u32, res_ptr| {
                let (errno, res) = clock_res_get(&this, mem, ClockId(clock))?;
                mem.write(res_ptr, res)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(clock_time_get) = self.clock_time_get {
            let this = this.clone();
            result.clock_time_get = Some(store.add_func(move |mem: &mut MemoryView, clock: u32, precision: u64, time_ptr| {
                let (errno, time) = clock_time_get(&this, mem, ClockId(clock), Timestamp { nanos: precision })?;
                mem.write(time_ptr, time)?;
                return Ok(errno.0 as u32);
            }).func());
        }

        if let Some(fd_advise) = self.fd_advise {
            let this = this.clone();
            result.fd_advise = Some(store.add_func(move |mem: &mut MemoryView, fd, offset, len, advice: u32| {
                let errno = fd_advise(&this, mem, Fd(fd), FileSize(offset), FileSize(len), Advice(advice as u8))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_allocate) = self.fd_allocate {
            let this = this.clone();
            result.fd_allocate = Some(store.add_func(move |mem: &mut MemoryView, fd, offset, len| {
                let errno = fd_allocate(&this, mem, Fd(fd), FileSize(offset), FileSize(len))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_close) = self.fd_close {
            let this = this.clone();
            result.fd_close = Some(store.add_func(move |mem: &mut MemoryView, fd| {
                let errno = fd_close(&this, mem, Fd(fd))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_datasync) = self.fd_datasync {
            let this = this.clone();
            result.fd_datasync = Some(store.add_func(move |mem: &mut MemoryView, fd| {
                let errno = fd_datasync(&this, mem, Fd(fd))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_fdstat_get) = self.fd_fdstat_get {
            let this = this.clone();
            result.fd_fdstat_get = Some(store.add_func(move |mem: &mut MemoryView, fd, fdstat_ptr| {
                let (errno, fdstat) = fd_fdstat_get(&this, mem, Fd(fd))?;
                mem.write(fdstat_ptr, fdstat)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_fdstat_set_flags) = self.fd_fdstat_set_flags {
            let this = this.clone();
            result.fd_fdstat_set_flags = Some(store.add_func(move |mem: &mut MemoryView, fd, flags: u32| {
                let errno = fd_fdstat_set_flags(&this, mem, Fd(fd), FdFlags(flags as u16))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_fdstat_set_rights) = self.fd_fdstat_set_rights {
            let this = this.clone();
            result.fd_fdstat_set_rights = Some(store.add_func(move |mem: &mut MemoryView, fd, rights_base, rights_inheriting| {
                let errno = fd_fdstat_set_rights(&this, mem, Fd(fd), Rights(rights_base), Rights(rights_inheriting))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_filestat_get) = self.fd_filestat_get {
            let this = this.clone();
            result.fd_filestat_get = Some(store.add_func(move |mem: &mut MemoryView, fd, stat_ptr| {
                let (errno, stat) = fd_filestat_get(&this, mem, Fd(fd))?;
                mem.write(stat_ptr, stat)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_filestat_set_size) = self.fd_filestat_set_size {
            let this = this.clone();
            result.fd_filestat_set_size = Some(store.add_func(move |mem: &mut MemoryView, fd, size| {
                let errno = fd_filestat_set_size(&this, mem, Fd(fd), FileSize(size))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_filestat_set_times) = self.fd_filestat_set_times {
            let this = this.clone();
            result.fd_filestat_set_times = Some(store.add_func(move |mem: &mut MemoryView, fd, accessed, modified, fst_flags: u32| {
                let errno = fd_filestat_set_times(&this, mem, Fd(fd), Timestamp { nanos: accessed }, Timestamp { nanos: modified }, FstFlags(fst_flags as u16))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_pread) = self.fd_pread {
            let this = this.clone();
            result.fd_pread = Some(store.add_func(move |mem: &mut MemoryView, fd, iovs, iovs_len, offset, nread_ptr| {
                let (errno, nread) = fd_pread(&this, mem, Fd(fd), iovs, iovs_len, FileSize(offset))?;
                mem.write(nread_ptr, nread)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_prestat_get) = self.fd_prestat_get {
            let this = this.clone();
            result.fd_prestat_get = Some(store.add_func(move |mem: &mut MemoryView, fd, prestat_ptr| {
                let (errno, prestat) = fd_prestat_get(&this, mem, Fd(fd))?;
                mem.write(prestat_ptr, prestat)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_prestat_dir_name) = self.fd_prestat_dir_name {
            let this = this.clone();
            result.fd_prestat_dir_name = Some(store.add_func(move |mem: &mut MemoryView, fd, path, path_len| {
                let errno = fd_prestat_dir_name(&this, mem, Fd(fd), path, path_len)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_pwrite) = self.fd_pwrite {
            let this = this.clone();
            result.fd_pwrite = Some(store.add_func(move |mem: &mut MemoryView, fd, iovs, iovs_len, offset, nwritten_ptr| {
                let (errno, nwritten) = fd_pwrite(&this, mem, Fd(fd), iovs, iovs_len, FileSize(offset))?;
                mem.write(nwritten_ptr, nwritten)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_read) = self.fd_read {
            let this = this.clone();
            result.fd_read = Some(store.add_func(move |mem: &mut MemoryView, fd, iovs, iovs_len, nread_ptr| {
                let (errno, nread) = fd_read(&this, mem, Fd(fd), iovs, iovs_len)?;
                mem.write(nread_ptr, nread)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_readdir) = self.fd_readdir {
            let this = this.clone();
            result.fd_readdir = Some(store.add_func(move |mem: &mut MemoryView, fd, buf, buf_len, cookie, bufused_ptr| {
                let (errno, bufused) = fd_readdir(&this, mem, Fd(fd), buf, buf_len, DirCookie(cookie))?;
                mem.write(bufused_ptr, bufused)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_renumber) = self.fd_renumber {
            let this = this.clone();
            result.fd_renumber = Some(store.add_func(move |mem: &mut MemoryView, from, to| {
                let errno = fd_renumber(&this, mem, Fd(from), Fd(to))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_seek) = self.fd_seek {
            let this = this.clone();
            result.fd_seek = Some(store.add_func(move |mem: &mut MemoryView, fd, delta, whence: u32, newoffset_ptr| {
                let (errno, newoffset) = fd_seek(&this, mem, Fd(fd), FileDelta(delta), Whence(whence as u8))?;
                mem.write(newoffset_ptr, newoffset)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_sync) = self.fd_sync {
            let this = this.clone();
            result.fd_sync = Some(store.add_func(move |mem: &mut MemoryView, fd| {
                let errno = fd_sync(&this, mem, Fd(fd))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_tell) = self.fd_tell {
            let this = this.clone();
            result.fd_tell = Some(store.add_func(move |mem: &mut MemoryView, fd, offset_ptr| {
                let (errno, offset) = fd_tell(&this, mem, Fd(fd))?;
                mem.write(offset_ptr, offset)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(fd_write) = self.fd_write {
            let this = this.clone();
            result.fd_write = Some(store.add_func(move |mem: &mut MemoryView, fd, iovs, iovs_len, nwritten_ptr| {
                let (errno, nwritten) = fd_write(&this, mem, Fd(fd), iovs, iovs_len)?;
                mem.write(nwritten_ptr, nwritten)?;
                return Ok(errno.0 as u32);
            }).func());
        }

        if let Some(path_create_directory) = self.path_create_directory {
            let this = this.clone();
            result.path_create_directory = Some(store.add_func(move |mem: &mut MemoryView, fd, path, path_len| {
                let errno = path_create_directory(&this, mem, Fd(fd), path, path_len)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(path_filestat_get) = self.path_filestat_get {
            let this = this.clone();
            result.path_filestat_get = Some(store.add_func(move |mem: &mut MemoryView, fd: u32, flags: u32, path: WasmPtr<u8>, path_len: WasmSize, filestat_ptr| {
                let (errno, filestat) = path_filestat_get(&this, mem, Fd(fd), LookupFlags(flags), path, path_len)?;
                mem.write(filestat_ptr, filestat)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(path_filestat_set_times) = self.path_filestat_set_times {
            let this = this.clone();
            result.path_filestat_set_times = Some(store.add_func(move |mem: &mut MemoryView, fd, flags, path, path_len, accessed, modified, fst_flags: u32| {
                let errno = path_filestat_set_times(&this, mem, Fd(fd), LookupFlags(flags), path, path_len, Timestamp { nanos: accessed }, Timestamp { nanos: modified }, FstFlags(fst_flags as u16))?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(path_link) = self.path_link {
            let this = this.clone();
            result.path_link = Some(store.add_func(move |mem: &mut MemoryView, old_fd, old_flags, old_path, old_path_len, new_fd, new_path, new_path_len| {
                let errno = path_link(&this, mem, Fd(old_fd), LookupFlags(old_flags), old_path, old_path_len, Fd(new_fd), new_path, new_path_len)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(path_open) = self.path_open {
            let this = this.clone();
            result.path_open = Some(store.add_func(move |mem: &mut MemoryView, fd, flags, path, path_len, open_flags: u32, rights_base, rights_inheriting, fd_flags: u32, fd_ptr| {
                let (errno, fd) = path_open(&this, mem, Fd(fd), LookupFlags(flags), path, path_len, OpenFlags(open_flags as u16), Rights(rights_base), Rights(rights_inheriting), FdFlags(fd_flags as u16))?;
                mem.write(fd_ptr, fd)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(path_readlink) = self.path_readlink {
            let this = this.clone();
            result.path_readlink = Some(store.add_func(move |mem: &mut MemoryView, fd, path, path_len, buf, buf_len, bufused_ptr| {
                let (errno, bufused) = path_readlink(&this, mem, Fd(fd), path, path_len, buf, buf_len)?;
                mem.write(bufused_ptr, bufused)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(path_remove_directory) = self.path_remove_directory {
            let this = this.clone();
            result.path_remove_directory = Some(store.add_func(move |mem: &mut MemoryView, fd, path, path_len| {
                let errno = path_remove_directory(&this, mem, Fd(fd), path, path_len)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(path_rename) = self.path_rename {
            let this = this.clone();
            result.path_rename = Some(store.add_func(move |mem: &mut MemoryView, old_base: u32, old_path: WasmPtr<u8>, old_path_len: WasmSize, new_base: u32, new_path: WasmPtr<u8>, new_path_len: WasmSize| {
                let errno = path_rename(&this, mem, Fd(old_base), old_path, old_path_len, Fd(new_base), new_path, new_path_len)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(path_symlink) = self.path_symlink {
            let this = this.clone();
            result.path_symlink = Some(store.add_func(move |mem: &mut MemoryView, old_path, old_path_len, base, new_path, new_path_len| {
                let errno = path_symlink(&this, mem, old_path, old_path_len, Fd(base), new_path, new_path_len)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(path_unlink_file) = self.path_unlink_file {
            let this = this.clone();
            result.path_unlink_file = Some(store.add_func(move |mem: &mut MemoryView, base, path, path_len| {
                let errno = path_unlink_file(&this, mem, Fd(base), path, path_len)?;
                return Ok(errno.0 as u32);
            }).func());
        }

        if let Some(poll_oneoff) = self.poll_oneoff {
            let this = this.clone();
            result.poll_oneoff = Some(store.add_func(move |mem: &mut MemoryView, subs, out, num_subs, numevents_ptr| {
                let (errno, numevents) = poll_oneoff(&this, mem, subs, out, num_subs)?;
                mem.write(numevents_ptr, numevents)?;
                return Ok(errno.0 as u32);
            }).func());
        }

        if let Some(proc_exit) = self.proc_exit {
            let this = this.clone();
            result.proc_exit = Some(store.add_func(move |mem: &mut MemoryView, exit_code| {
                proc_exit(&this, mem, ExitCode(exit_code))
            }).func());
        }
        if let Some(proc_raise) = self.proc_raise {
            let this = this.clone();
            result.proc_raise = Some(store.add_func(move |mem: &mut MemoryView, signal: u32| {
                let errno = proc_raise(&this, mem, Signal(signal as u8))?;
                return Ok(errno.0 as u32);
            }).func());
        }

        if let Some(sched_yield) = self.sched_yield {
            let this = this.clone();
            result.sched_yield = Some(store.add_func(move |mem: &mut MemoryView| {
                let errno = sched_yield(&this, mem)?;
                return Ok(errno.0 as u32);
            }).func());
        }

        if let Some(random_get) = self.random_get {
            let this = this.clone();
            result.random_get = Some(store.add_func(move |mem: &mut MemoryView, buf, buf_len| {
                let errno = random_get(&this, mem, buf, buf_len)?;
                return Ok(errno.0 as u32);
            }).func());
        }

        if let Some(sock_accept) = self.sock_accept {
            let this = this.clone();
            result.sock_accept = Some(store.add_func(move |mem: &mut MemoryView, fd, flags: u32, outfd_ptr| {
                let (errno, outfd) = sock_accept(&this, mem, Fd(fd), FdFlags(flags as u16))?;
                mem.write(outfd_ptr, outfd)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(sock_recv) = self.sock_recv {
            let this = this.clone();
            result.sock_recv = Some(store.add_func(move |mem: &mut MemoryView, fd, iovs, iovs_len, flags: u32, nrecv_ptr, outflags_ptr| {
                let (errno, nrecv, outflags) = sock_recv(&this, mem, Fd(fd), iovs, iovs_len, SockRecvInFlags(flags as u16))?;
                mem.write(nrecv_ptr, nrecv)?;
                mem.write(outflags_ptr, outflags)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(sock_send) = self.sock_send {
            let this = this.clone();
            result.sock_send = Some(store.add_func(move |mem: &mut MemoryView, fd, iovs, iovs_len, flags: u32, nsent_ptr| {
                let (errno, nsent) = sock_send(&this, mem, Fd(fd), iovs, iovs_len, SockSendFlags(flags as u16))?;
                mem.write(nsent_ptr, nsent)?;
                return Ok(errno.0 as u32);
            }).func());
        }
        if let Some(sock_shutdown) = self.sock_shutdown {
            let this = this.clone();
            result.sock_shutdown = Some(store.add_func(move |mem: &mut MemoryView, fd, flags: u32| {
                let errno = sock_shutdown(&this, mem, Fd(fd), SockShutdownFlags(flags as u8))?;
                return Ok(errno.0 as u32);
            }).func());
        }

        return result;
    }
}

impl StoreFuncs {
    pub fn add_imports(&self, imports: &mut Imports) {
        let module = "wasi_snapshot_preview1";

        if let Some(args_sizes_get) = self.args_sizes_get {
            imports.add(module, "args_sizes_get", args_sizes_get.into());
        }
        if let Some(args_get) = self.args_get {
            imports.add(module, "args_get", args_get.into());
        }

        if let Some(environ_sizes_get) = self.environ_sizes_get {
            imports.add(module, "environ_sizes_get", environ_sizes_get.into());
        }
        if let Some(environ_get) = self.environ_get {
            imports.add(module, "environ_get", environ_get.into());
        }

        if let Some(clock_res_get) = self.clock_res_get {
            imports.add(module, "clock_res_get", clock_res_get.into());
        }
        if let Some(clock_time_get) = self.clock_time_get {
            imports.add(module, "clock_time_get", clock_time_get.into());
        }

        if let Some(fd_advise) = self.fd_advise {
            imports.add(module, "fd_advise", fd_advise.into());
        }
        if let Some(fd_allocate) = self.fd_allocate {
            imports.add(module, "fd_allocate", fd_allocate.into());
        }
        if let Some(fd_close) = self.fd_close {
            imports.add(module, "fd_close", fd_close.into());
        }
        if let Some(fd_datasync) = self.fd_datasync {
            imports.add(module, "fd_datasync", fd_datasync.into());
        }
        if let Some(fd_fdstat_get) = self.fd_fdstat_get {
            imports.add(module, "fd_fdstat_get", fd_fdstat_get.into());
        }
        if let Some(fd_fdstat_set_flags) = self.fd_fdstat_set_flags {
            imports.add(module, "fd_fdstat_set_flags", fd_fdstat_set_flags.into());
        }
        if let Some(fd_fdstat_set_rights) = self.fd_fdstat_set_rights {
            imports.add(module, "fd_fdstat_set_rights", fd_fdstat_set_rights.into());
        }
        if let Some(fd_filestat_get) = self.fd_filestat_get {
            imports.add(module, "fd_filestat_get", fd_filestat_get.into());
        }
        if let Some(fd_filestat_set_size) = self.fd_filestat_set_size {
            imports.add(module, "fd_filestat_set_size", fd_filestat_set_size.into());
        }
        if let Some(fd_filestat_set_times) = self.fd_filestat_set_times {
            imports.add(module, "fd_filestat_set_times", fd_filestat_set_times.into());
        }
        if let Some(fd_pread) = self.fd_pread {
            imports.add(module, "fd_pread", fd_pread.into());
        }
        if let Some(fd_prestat_get) = self.fd_prestat_get {
            imports.add(module, "fd_prestat_get", fd_prestat_get.into());
        }
        if let Some(fd_prestat_dir_name) = self.fd_prestat_dir_name {
            imports.add(module, "fd_prestat_dir_name", fd_prestat_dir_name.into());
        }
        if let Some(fd_pwrite) = self.fd_pwrite {
            imports.add(module, "fd_pwrite", fd_pwrite.into());
        }
        if let Some(fd_read) = self.fd_read {
            imports.add(module, "fd_read", fd_read.into());
        }
        if let Some(fd_readdir) = self.fd_readdir {
            imports.add(module, "fd_readdir", fd_readdir.into());
        }
        if let Some(fd_renumber) = self.fd_renumber {
            imports.add(module, "fd_renumber", fd_renumber.into());
        }
        if let Some(fd_seek) = self.fd_seek {
            imports.add(module, "fd_seek", fd_seek.into());
        }
        if let Some(fd_sync) = self.fd_sync {
            imports.add(module, "fd_sync", fd_sync.into());
        }
        if let Some(fd_tell) = self.fd_tell {
            imports.add(module, "fd_tell", fd_tell.into());
        }
        if let Some(fd_write) = self.fd_write {
            imports.add(module, "fd_write", fd_write.into());
        }

        if let Some(path_create_directory) = self.path_create_directory {
            imports.add(module, "path_create_directory", path_create_directory.into());
        }
        if let Some(path_filestat_get) = self.path_filestat_get {
            imports.add(module, "path_filestat_get", path_filestat_get.into());
        }
        if let Some(path_filestat_set_times) = self.path_filestat_set_times {
            imports.add(module, "path_filestat_set_times", path_filestat_set_times.into());
        }
        if let Some(path_link) = self.path_link {
            imports.add(module, "path_link", path_link.into());
        }
        if let Some(path_open) = self.path_open {
            imports.add(module, "path_open", path_open.into());
        }
        if let Some(path_readlink) = self.path_readlink {
            imports.add(module, "path_readlink", path_readlink.into());
        }
        if let Some(path_remove_directory) = self.path_remove_directory {
            imports.add(module, "path_remove_directory", path_remove_directory.into());
        }
        if let Some(path_rename) = self.path_rename {
            imports.add(module, "path_rename", path_rename.into());
        }
        if let Some(path_symlink) = self.path_symlink {
            imports.add(module, "path_symlink", path_symlink.into());
        }
        if let Some(path_unlink_file) = self.path_unlink_file {
            imports.add(module, "path_unlink_file", path_unlink_file.into());
        }

        if let Some(poll_oneoff) = self.poll_oneoff {
            imports.add(module, "poll_oneoff", poll_oneoff.into());
        }

        if let Some(proc_exit) = self.proc_exit {
            imports.add(module, "proc_exit", proc_exit.into());
        }
        if let Some(proc_raise) = self.proc_raise {
            imports.add(module, "proc_raise", proc_raise.into());
        }

        if let Some(sched_yield) = self.sched_yield {
            imports.add(module, "sched_yield", sched_yield.into());
        }

        if let Some(random_get) = self.random_get {
            imports.add(module, "random_get", random_get.into());
        }

        if let Some(sock_accept) = self.sock_accept {
            imports.add(module, "sock_accept", sock_accept.into());
        }
        if let Some(sock_recv) = self.sock_recv {
            imports.add(module, "sock_recv", sock_recv.into());
        }
        if let Some(sock_send) = self.sock_send {
            imports.add(module, "sock_send", sock_send.into());
        }
        if let Some(sock_shutdown) = self.sock_shutdown {
            imports.add(module, "sock_shutdown", sock_shutdown.into());
        }
    }
}

