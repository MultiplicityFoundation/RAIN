//! Board registry — maps USB VID/PID to known board names and architectures.

/// Information about a known board.
#[derive(Debug, Clone)]
pub struct BoardInfo {
    pub vid: u16,
    pub pid: u16,
    pub name: &'static str,
    pub architecture: Option<&'static str>,
}

/// Known USB VID/PID to board mappings.
/// VID 0x0483 = STMicroelectronics, 0x2341 = Arduino, 0x10c4 = Silicon Labs.
const KNOWN_BOARDS: &[BoardInfo] = &[
    BoardInfo {
        vid: 0x0483,
        pid: 0x374b,
        name: "nucleo-f401re",
        architecture: Some("ARM Cortex-M4"),
    },
    BoardInfo {
        vid: 0x0483,
        pid: 0x3748,
        name: "nucleo-f411re",
        architecture: Some("ARM Cortex-M4"),
    },
    BoardInfo {
        vid: 0x2341,
        pid: 0x0043,
        name: "arduino-uno",
        architecture: Some("AVR ATmega328P"),
    },
    BoardInfo {
        vid: 0x2341,
        pid: 0x0078,
        name: "arduino-uno",
        architecture: Some("Arduino Uno Q / ATmega328P"),
    },
    BoardInfo {
        vid: 0x2341,
        pid: 0x0042,
        name: "arduino-mega",
        architecture: Some("AVR ATmega2560"),
    },
    BoardInfo {
        vid: 0x10c4,
        pid: 0xea60,
        name: "cp2102",
        architecture: Some("USB-UART bridge"),
    },
    BoardInfo {
        vid: 0x10c4,
        pid: 0xea70,
        name: "cp2102n",
        architecture: Some("USB-UART bridge"),
    },
    // ESP32 dev boards often use CH340 USB-UART
    BoardInfo {
        vid: 0x1a86,
        pid: 0x7523,
        name: "esp32",
        architecture: Some("ESP32 (CH340)"),
    },
    BoardInfo {
        vid: 0x1a86,
        pid: 0x55d4,
        name: "esp32",
        architecture: Some("ESP32 (CH340)"),
    },
];

/// Look up a board by VID and PID.
pub fn lookup_board(vid: u16, pid: u16) -> Option<&'static BoardInfo> {
    KNOWN_BOARDS.iter().find(|b| b.vid == vid && b.pid == pid)
}

/// Return all known board entries.
pub fn known_boards() -> &'static [BoardInfo] {
    KNOWN_BOARDS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_nucleo_f401re() {
        let b = lookup_board(0x0483, 0x374b).unwrap();
        assert_eq!(b.name, "nucleo-f401re");
        assert_eq!(b.architecture, Some("ARM Cortex-M4"));
    }

    #[test]
    fn lookup_nucleo_f411re() {
        let b = lookup_board(0x0483, 0x3748).unwrap();
        assert_eq!(b.name, "nucleo-f411re");
        assert_eq!(b.architecture, Some("ARM Cortex-M4"));
    }

    #[test]
    fn lookup_arduino_uno() {
        let b = lookup_board(0x2341, 0x0043).unwrap();
        assert_eq!(b.name, "arduino-uno");
    }

    #[test]
    fn lookup_arduino_uno_q() {
        let b = lookup_board(0x2341, 0x0078).unwrap();
        assert_eq!(b.name, "arduino-uno");
        assert!(b.architecture.unwrap().contains("Uno Q"));
    }

    #[test]
    fn lookup_arduino_mega() {
        let b = lookup_board(0x2341, 0x0042).unwrap();
        assert_eq!(b.name, "arduino-mega");
    }

    #[test]
    fn lookup_cp2102_uart_bridge() {
        let b = lookup_board(0x10c4, 0xea60).unwrap();
        assert_eq!(b.name, "cp2102");
        assert!(b.architecture.unwrap().contains("USB-UART"));
    }

    #[test]
    fn lookup_esp32_ch340() {
        let b = lookup_board(0x1a86, 0x7523).unwrap();
        assert_eq!(b.name, "esp32");
        assert!(b.architecture.unwrap().contains("CH340"));
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(lookup_board(0x0000, 0x0000).is_none());
    }

    #[test]
    fn lookup_vid_match_pid_mismatch_returns_none() {
        // STM VID but wrong PID
        assert!(lookup_board(0x0483, 0xFFFF).is_none());
    }

    #[test]
    fn known_boards_not_empty() {
        assert!(!known_boards().is_empty());
    }

    #[test]
    fn all_known_boards_have_names_and_architectures() {
        for board in known_boards() {
            assert!(
                !board.name.is_empty(),
                "Board with VID={:#06x} PID={:#06x} has empty name",
                board.vid,
                board.pid
            );
            assert!(
                board.architecture.is_some(),
                "Board {} has no architecture",
                board.name
            );
        }
    }

    #[test]
    fn no_duplicate_vid_pid_pairs() {
        let boards = known_boards();
        for (i, a) in boards.iter().enumerate() {
            for (j, b) in boards.iter().enumerate() {
                if i != j {
                    assert!(
                        !(a.vid == b.vid && a.pid == b.pid),
                        "Duplicate VID/PID: {:#06x}/{:#06x} for {} and {}",
                        a.vid,
                        a.pid,
                        a.name,
                        b.name
                    );
                }
            }
        }
    }
}
