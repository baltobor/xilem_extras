//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Mock data for the gallery demos.
//!
//! I love riding the bike. This is my honour to the community, and outdoor sports.

use xilem_extras::{Identifiable, TreeNode, ListItem, TableRow, CellValue};

/// A node in the file tree.
#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

impl FileNode {
    pub fn file(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            is_dir: false,
            children: Vec::new(),
        }
    }

    pub fn dir(name: impl Into<String>, path: impl Into<String>, children: Vec<FileNode>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            is_dir: true,
            children,
        }
    }
}

impl Identifiable for FileNode {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.path.clone()
    }
}

impl TreeNode for FileNode {
    fn children(&self) -> &[Self] {
        &self.children
    }

    fn is_expandable(&self) -> bool {
        self.is_dir
    }

    fn label(&self) -> &str {
        &self.name
    }
}

/// Creates a mock file tree.
pub fn mock_file_tree() -> FileNode {
    FileNode::dir(
        "project",
        ".",
        vec![
            FileNode::dir(
                "src",
                "src",
                vec![
                    FileNode::file("main.rs", "src/main.rs"),
                    FileNode::file("lib.rs", "src/lib.rs"),
                    FileNode::dir(
                        "components",
                        "src/components",
                        vec![
                            FileNode::file("mod.rs", "src/components/mod.rs"),
                            FileNode::file("button.rs", "src/components/button.rs"),
                            FileNode::file("tree.rs", "src/components/tree.rs"),
                        ],
                    ),
                ],
            ),
            FileNode::dir(
                "tests",
                "tests",
                vec![
                    FileNode::file("integration.rs", "tests/integration.rs"),
                ],
            ),
            FileNode::file("Cargo.toml", "Cargo.toml"),
            FileNode::file("README.md", "README.md"),
        ],
    )
}

/// A community member for the list demo.
#[derive(Debug, Clone)]
pub struct Contact {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub favorite: bool,
}

impl Identifiable for Contact {
    type Id = u64;

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl ListItem for Contact {
    fn label(&self) -> &str {
        &self.name
    }

    fn subtitle(&self) -> Option<&str> {
        Some(&self.email)
    }
}

/// Creates mock community members.
pub fn mock_contacts() -> Vec<Contact> {
    vec![
        Contact {
            id: 1,
            name: "Luna Park".into(),
            email: "luna@email.tld".into(),
            favorite: true,
        },
        Contact {
            id: 2,
            name: "Felix Walker".into(),
            email: "felix@email.tld".into(),
            favorite: false,
        },
        Contact {
            id: 3,
            name: "Maya Sunshine".into(),
            email: "maya@email.tld".into(),
            favorite: true,
        },
        Contact {
            id: 4,
            name: "River Stone".into(),
            email: "river@email.tld".into(),
            favorite: false,
        },
        Contact {
            id: 5,
            name: "Sage Meadow".into(),
            email: "sage@email.tld".into(),
            favorite: false,
        },
    ]
}

/// A cyclist for the table demo - celebrating active mobility.
#[derive(Debug, Clone)]
pub struct Cyclist {
    pub id: u64,
    pub name: String,
    pub route: String,
    pub distance_km: f64,
    pub joy_level: i64,
}

impl Identifiable for Cyclist {
    type Id = u64;

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl TableRow for Cyclist {
    fn cell(&self, column_key: &str) -> CellValue {
        match column_key {
            "name" => CellValue::Text(self.name.clone()),
            "route" => CellValue::Text(self.route.clone()),
            "distance_km" => CellValue::Float(self.distance_km, 1),
            "joy_level" => CellValue::Integer(self.joy_level),
            _ => CellValue::Empty,
        }
    }
}

/// Creates mock cyclists - happy people on two wheels.
pub fn mock_cyclists() -> Vec<Cyclist> {
    vec![
        Cyclist {
            id: 1,
            name: "Luna Park".into(),
            route: "Riverside Trail".into(),
            distance_km: 25.5,
            joy_level: 10,
        },
        Cyclist {
            id: 2,
            name: "Felix Walker".into(),
            route: "Mountain Loop".into(),
            distance_km: 42.0,
            joy_level: 9,
        },
        Cyclist {
            id: 3,
            name: "Maya Sunshine".into(),
            route: "Beach Path".into(),
            distance_km: 15.2,
            joy_level: 10,
        },
        Cyclist {
            id: 4,
            name: "River Stone".into(),
            route: "Forest Circuit".into(),
            distance_km: 33.7,
            joy_level: 8,
        },
        Cyclist {
            id: 5,
            name: "Sage Meadow".into(),
            route: "City Greenway".into(),
            distance_km: 18.9,
            joy_level: 9,
        },
        Cyclist {
            id: 6,
            name: "Willow Creek".into(),
            route: "Lakeside Route".into(),
            distance_km: 28.4,
            joy_level: 10,
        },
        Cyclist {
            id: 7,
            name: "Jack Pearse".into(),
            route: "Vennbahnweg".into(),
            distance_km: 125.0,
            joy_level: 10,
        },
    ]
}

/// First names for generating cyclists.
const FIRST_NAMES: &[&str] = &[
    "Luna", "Felix", "Maya", "River", "Sage", "Willow", "Jack", "Emma", "Liam", "Olivia",
    "Noah", "Ava", "Ethan", "Sophia", "Mason", "Isabella", "Logan", "Mia", "Lucas", "Charlotte",
    "Alexander", "Amelia", "Benjamin", "Harper", "Elijah", "Evelyn", "James", "Abigail", "William", "Emily",
];

/// Last names for generating cyclists.
const LAST_NAMES: &[&str] = &[
    "Park", "Walker", "Sunshine", "Stone", "Meadow", "Creek", "Pearse", "Smith", "Johnson", "Brown",
    "Davis", "Miller", "Wilson", "Moore", "Taylor", "Anderson", "Thomas", "Jackson", "White", "Harris",
    "Martin", "Thompson", "Garcia", "Martinez", "Robinson", "Clark", "Rodriguez", "Lewis", "Lee", "Hall",
];

/// Routes for generating cyclists.
const ROUTES: &[&str] = &[
    "Riverside Trail", "Mountain Loop", "Beach Path", "Forest Circuit", "City Greenway",
    "Lakeside Route", "Vennbahnweg", "Alpine Pass", "Coastal Highway", "Valley Road",
    "Sunset Boulevard", "Harbor Bridge", "Canal Path", "Vineyard Tour", "Historic Route",
    "Meadow Lane", "Hilltop Circuit", "River Crossing", "Woodland Trail", "Seaside Promenade",
];

/// Creates a large dataset of cyclists for performance testing.
///
/// Generates `count` cyclists with pseudo-random but deterministic data.
pub fn mock_cyclists_large(count: usize) -> Vec<Cyclist> {
    (0..count)
        .map(|i| {
            let id = i as u64 + 1;
            // Simple deterministic "randomization" based on index
            let first_idx = (i * 7 + 3) % FIRST_NAMES.len();
            let last_idx = (i * 11 + 5) % LAST_NAMES.len();
            let route_idx = (i * 13 + 7) % ROUTES.len();

            let name = format!("{} {}", FIRST_NAMES[first_idx], LAST_NAMES[last_idx]);
            let route = ROUTES[route_idx].to_string();
            let distance_km = 10.0 + ((i * 17) % 200) as f64 + (i % 10) as f64 * 0.1;
            let joy_level = ((i * 3) % 10 + 1) as i64;

            Cyclist {
                id,
                name,
                route,
                distance_km,
                joy_level,
            }
        })
        .collect()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_node_identifiable() {
        let node = FileNode::file("test.rs", "src/test.rs");
        assert_eq!(node.id(), "src/test.rs");
    }

    #[test]
    fn file_node_tree_node() {
        let dir = FileNode::dir(
            "src",
            "src",
            vec![FileNode::file("main.rs", "src/main.rs")],
        );
        assert!(dir.is_expandable());
        assert_eq!(dir.children().len(), 1);
        assert_eq!(dir.label(), "src");
    }

    #[test]
    fn file_leaf_not_expandable() {
        let file = FileNode::file("main.rs", "src/main.rs");
        assert!(!file.is_expandable());
    }

    #[test]
    fn contact_identifiable() {
        let contact = Contact {
            id: 42,
            name: "Test".into(),
            email: "test@email.tld".into(),
            favorite: false,
        };
        assert_eq!(contact.id(), 42);
    }

    #[test]
    fn contact_list_item() {
        let contact = Contact {
            id: 1,
            name: "Luna".into(),
            email: "luna@email.tld".into(),
            favorite: false,
        };
        assert_eq!(contact.label(), "Luna");
        assert_eq!(contact.subtitle(), Some("luna@email.tld"));
    }

    #[test]
    fn cyclist_table_row() {
        let cyclist = Cyclist {
            id: 1,
            name: "Test".into(),
            route: "Trail".into(),
            distance_km: 25.5,
            joy_level: 10,
        };

        assert_eq!(cyclist.cell("name"), CellValue::Text("Test".into()));
        assert_eq!(cyclist.cell("route"), CellValue::Text("Trail".into()));
        assert_eq!(cyclist.cell("distance_km"), CellValue::Float(25.5, 1));
        assert_eq!(cyclist.cell("joy_level"), CellValue::Integer(10));
        assert_eq!(cyclist.cell("unknown"), CellValue::Empty);
    }
}
