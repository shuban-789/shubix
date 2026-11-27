use crate::fs::structs::Node;
use crate::fs::structs::NodeKind;

// fs initialization to follow FHS standard
static BIN_CHILDREN: [Node; 0] = [];
static SBIN_CHILDREN: [Node; 0] = [];
static BOOT_CHILDREN: [Node; 0] = [];
static DEV_CHILDREN: [Node; 0] = [];
static ETC_CHILDREN: [Node; 0] = [];
static LIB_CHILDREN: [Node; 0] = [];
static MEDIA_CHILDREN: [Node; 0] = [];
static MNT_CHILDREN: [Node; 0] = [];
static OPT_CHILDREN: [Node; 0] = [];
static RUN_CHILDREN: [Node; 0] = [];
static SRV_CHILDREN: [Node; 0] = [];
static TMP_CHILDREN: [Node; 0] = [];
static USR_CHILDREN: [Node; 0] = [];
static VAR_CHILDREN: [Node; 0] = [];

static ROOT_CHILDREN: [Node; 14] = [
    Node {
        name: "bin",
        kind: NodeKind::Directory(&BIN_CHILDREN),
    },
    Node {
        name: "sbin",
        kind: NodeKind::Directory(&SBIN_CHILDREN),
    },
    Node {
        name: "boot",
        kind: NodeKind::Directory(&BOOT_CHILDREN),
    },
    Node {
        name: "dev",
        kind: NodeKind::Directory(&DEV_CHILDREN),
    },
    Node {
        name: "etc",
        kind: NodeKind::Directory(&ETC_CHILDREN),
    },
    Node {
        name: "lib",
        kind: NodeKind::Directory(&LIB_CHILDREN),
    },
    Node {
        name: "media",
        kind: NodeKind::Directory(&MEDIA_CHILDREN),
    },
    Node {
        name: "mnt",
        kind: NodeKind::Directory(&MNT_CHILDREN),
    },
    Node {
        name: "opt",
        kind: NodeKind::Directory(&OPT_CHILDREN),
    },
    Node {
        name: "run",
        kind: NodeKind::Directory(&RUN_CHILDREN),
    },
    Node {
        name: "srv",
        kind: NodeKind::Directory(&SRV_CHILDREN),
    },
    Node {
        name: "tmp",
        kind: NodeKind::Directory(&TMP_CHILDREN),
    },
    Node {
        name: "usr",
        kind: NodeKind::Directory(&USR_CHILDREN),
    },
    Node {
        name: "var",
        kind: NodeKind::Directory(&VAR_CHILDREN),
    },
];

pub static ROOT: Node = Node {
    name: "/",
    kind: NodeKind::Directory(&ROOT_CHILDREN),
};
