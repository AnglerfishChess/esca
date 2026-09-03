//! The versioned contract between the extractor and the net: which groups a
//! feature vector carries, in which order, and how wide each is.

use core::fmt;

use crate::variant::Variant;

/// How many features the widest schema may name.
const MAX_FEATURES: usize = 256;

/// One named value inside a group.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FeatureSpec {
    /// The feature's name, unique within its group.
    pub name: &'static str,
    /// Where the feature starts inside the group, in values.
    pub offset: usize,
    /// How many values the feature occupies.
    pub width: usize,
    /// The encoding kind and its scale, as the canonical text spells it.
    pub encoding: &'static str,
    /// The variants the feature is defined for; empty means every variant.
    pub variants: &'static [&'static str],
}

impl FeatureSpec {
    /// Whether the feature's definition holds under the variant named `name`.
    #[inline]
    pub fn defined_for(&self, name: &str) -> bool {
        self.variants.is_empty() || self.variants.contains(&name)
    }
}

/// A named, independently versioned block of features.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GroupSpec {
    /// The group's name.
    pub name: &'static str,
    /// The group's version, bumped when its features change.
    pub version: u16,
    /// How many values the group occupies.
    pub width: usize,
    /// The features, in the order they are written.
    pub features: &'static [FeatureSpec],
}

/// A subset of a schema's groups.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct GroupSet(u16);

impl GroupSet {
    /// No groups.
    pub const EMPTY: GroupSet = GroupSet(0);

    /// The set holding only the group at `index`.
    #[inline]
    pub const fn only(index: usize) -> GroupSet {
        GroupSet(1 << index)
    }

    /// Whether the group at `index` is a member.
    #[inline]
    pub const fn contains(self, index: usize) -> bool {
        self.0 & (1 << index) != 0
    }

    /// Adds the group at `index`.
    #[inline]
    pub fn insert(&mut self, index: usize) {
        self.0 |= 1 << index;
    }

    /// Removes the group at `index`.
    #[inline]
    pub fn remove(&mut self, index: usize) {
        self.0 &= !(1 << index);
    }

    /// Whether no group is a member.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// How many groups are members.
    #[inline]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }
}

/// A 128-bit hash over a schema's canonical text.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SchemaId([u8; 16]);

impl SchemaId {
    /// The hash bytes.
    #[inline]
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

impl fmt::Display for SchemaId {
    /// 32 lower-case hex digits.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// A subset of a schema's features, by group and name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FeatureSet {
    schema: &'static Schema,
    bits: [u64; MAX_FEATURES / 64],
}

impl FeatureSet {
    /// Whether the schema names `feature` in `group` and it is a member.
    pub fn contains(&self, group: &str, feature: &str) -> bool {
        match self.schema.feature_index(group, feature) {
            Some(index) => self.bits[index / 64] & (1 << (index % 64)) != 0,
            None => false,
        }
    }

    /// The members, as group and feature names, in schema order.
    pub fn names(&self) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        self.schema
            .groups
            .iter()
            .flat_map(|group| group.features.iter().map(move |f| (group.name, f.name)))
            .enumerate()
            .filter(|(index, _)| self.bits[index / 64] & (1 << (index % 64)) != 0)
            .map(|(_, names)| names)
    }
}

/// An ordered list of feature groups: what a feature vector carries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Schema {
    semver: &'static str,
    groups: &'static [GroupSpec],
}

impl Schema {
    /// The v1 schema: the fourteen groups of `features.md`, in its order.
    pub fn v1() -> &'static Schema {
        &V1
    }

    /// The schema's semantic version.
    #[inline]
    pub fn semver(&self) -> &'static str {
        self.semver
    }

    /// The width of every group together.
    pub fn width(&self) -> usize {
        self.groups.iter().map(|group| group.width).sum()
    }

    /// The width of the selected groups.
    pub fn width_of(&self, groups: GroupSet) -> usize {
        self.groups
            .iter()
            .enumerate()
            .filter(|(index, _)| groups.contains(*index))
            .map(|(_, group)| group.width)
            .sum()
    }

    /// The groups, in the order they are written.
    #[inline]
    pub fn groups(&self) -> &'static [GroupSpec] {
        self.groups
    }

    /// The group of that name.
    pub fn group(&self, name: &str) -> Option<&'static GroupSpec> {
        self.groups.iter().find(|group| group.name == name)
    }

    /// Where the group of that name sits in the schema order.
    pub fn group_index(&self, name: &str) -> Option<usize> {
        self.groups.iter().position(|group| group.name == name)
    }

    /// Every group.
    pub fn all(&self) -> GroupSet {
        let mut set = GroupSet::EMPTY;
        for index in 0..self.groups.len() {
            set.insert(index);
        }
        set
    }

    /// The named groups, or `None` when a name is not one of the schema's.
    pub fn group_set(&self, names: &[&str]) -> Option<GroupSet> {
        let mut set = GroupSet::EMPTY;
        for name in names {
            set.insert(self.group_index(name)?);
        }
        Some(set)
    }

    /// The canonical text `id` hashes: one line per group, then one indented
    /// line per feature.
    pub fn canonical(&self) -> String {
        let mut out = String::new();
        for group in self.groups {
            out.push_str(&format!(
                "{}:{}:{}\n",
                group.name, group.version, group.width
            ));
            for feature in group.features {
                out.push_str(&format!(
                    "  {}:{}:{}\n",
                    feature.name, feature.width, feature.encoding
                ));
            }
        }
        out
    }

    /// The BLAKE3 hash of [`Schema::canonical`], truncated to 128 bits.
    pub fn id(&self) -> SchemaId {
        let digest = blake3::hash(self.canonical().as_bytes());
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&digest.as_bytes()[..16]);
        SchemaId(bytes)
    }

    /// The features whose definitions hold under `variant`.
    pub fn features_for(&'static self, variant: &dyn Variant) -> FeatureSet {
        let name = variant.name();
        let mut bits = [0u64; MAX_FEATURES / 64];
        let mut index = 0;
        for group in self.groups {
            for feature in group.features {
                if feature.defined_for(name) {
                    bits[index / 64] |= 1 << (index % 64);
                }
                index += 1;
            }
        }
        FeatureSet { schema: self, bits }
    }

    /// How many features the schema names.
    pub fn feature_count(&self) -> usize {
        self.groups.iter().map(|group| group.features.len()).sum()
    }

    fn feature_index(&self, group: &str, feature: &str) -> Option<usize> {
        let mut index = 0;
        for spec in self.groups {
            for entry in spec.features {
                if spec.name == group && entry.name == feature {
                    return Some(index);
                }
                index += 1;
            }
        }
        None
    }
}

/// Fills in each feature's offset from the widths before it.
const fn with_offsets<const N: usize>(mut specs: [FeatureSpec; N]) -> [FeatureSpec; N] {
    let mut index = 0;
    let mut offset = 0;
    while index < N {
        specs[index].offset = offset;
        offset += specs[index].width;
        index += 1;
    }
    specs
}

/// Every variant.
const ANY: &[&str] = &[];
/// Classic chess only: the feature assumes the standard starting squares.
const CLASSIC_ONLY: &[&str] = &["chess"];

macro_rules! feature_specs {
    ($($name:literal, $width:literal, $encoding:literal, $variants:expr;)*) => {
        with_offsets([$(FeatureSpec {
            name: $name,
            offset: 0,
            width: $width,
            encoding: $encoding,
            variants: $variants,
        }),*])
    };
}

static PLACEMENT: [FeatureSpec; 12] = feature_specs! {
    "our_pawns", 64, "plane", ANY;
    "our_knights", 64, "plane", ANY;
    "our_bishops", 64, "plane", ANY;
    "our_rooks", 64, "plane", ANY;
    "our_queens", 64, "plane", ANY;
    "our_king", 64, "plane", ANY;
    "their_pawns", 64, "plane", ANY;
    "their_knights", 64, "plane", ANY;
    "their_bishops", 64, "plane", ANY;
    "their_rooks", 64, "plane", ANY;
    "their_queens", 64, "plane", ANY;
    "their_king", 64, "plane", ANY;
};

static STATE: [FeatureSpec; 6] = feature_specs! {
    "in_check", 1, "bit", ANY;
    "double_check", 1, "bit", ANY;
    "castle_rights", 4, "bits", ANY;
    "ep_available", 1, "bit", ANY;
    "ep_file", 8, "one-hot", ANY;
    "ep_capture_legal", 1, "bit", ANY;
};

static MATERIAL: [FeatureSpec; 9] = feature_specs! {
    "piece_count", 10, "count/8|4", ANY;
    "piece_count_diff", 5, "diff/4", ANY;
    "non_pawn_material", 2, "count/62", ANY;
    "material_balance", 1, "diff/20", ANY;
    "phase", 1, "ratio", ANY;
    "phase_bucket", 3, "one-hot", ANY;
    "both_queens", 1, "bit", ANY;
    "pawns_only", 1, "bit", ANY;
    "insufficient_material", 2, "bits", ANY;
};

static PAWNS: [FeatureSpec; 18] = feature_specs! {
    "pawn_count_by_file", 16, "count/3", ANY;
    "pawn_count_by_rank", 16, "count/8", ANY;
    "doubled_files", 16, "mask8", ANY;
    "isolated_files", 16, "mask8", ANY;
    "backward_files", 16, "mask8", ANY;
    "passed_files", 16, "mask8", ANY;
    "candidate_files", 16, "mask8", ANY;
    "passer_lead_rank", 16, "one-hot", ANY;
    "passer_protected", 2, "count/4", ANY;
    "passers_connected", 2, "bits", ANY;
    "passer_unstoppable", 2, "bits", ANY;
    "open_files", 8, "mask8", ANY;
    "semi_open_files_us", 8, "mask8", ANY;
    "semi_open_files_them", 8, "mask8", ANY;
    "pawn_islands", 2, "count/4", ANY;
    "defended_pawns", 2, "count/8", ANY;
    "levers", 2, "count/4", ANY;
    "rams", 1, "count/8", ANY;
};

static PIECES: [FeatureSpec; 17] = feature_specs! {
    "bishop_pair", 2, "bits", ANY;
    "bishops_by_square_colour", 4, "count/2", ANY;
    "opposite_coloured_bishops", 1, "bit", ANY;
    "pawns_on_bishop_colour", 2, "count/8", ANY;
    "rooks_connected_rank", 2, "bits", ANY;
    "rooks_connected_file", 2, "bits", ANY;
    "rooks_on_open_file", 2, "count/2", ANY;
    "rooks_on_semi_open_file", 2, "count/2", ANY;
    "rooks_on_relative_7th", 2, "count/2", ANY;
    "rook_behind_own_passer", 2, "count/2", ANY;
    "rook_behind_enemy_passer", 2, "count/2", ANY;
    "trapped_rook", 2, "bits", ANY;
    "minors_on_outpost", 2, "count/2", ANY;
    "outpost_squares_free", 2, "count/4", ANY;
    "knights_on_rim", 2, "count/2", ANY;
    "minors_undeveloped", 2, "count/4", CLASSIC_ONLY;
    "queen_developed", 2, "bits", CLASSIC_ONLY;
};

static KING: [FeatureSpec; 16] = feature_specs! {
    "king_file", 16, "one-hot", ANY;
    "king_rank", 16, "one-hot", ANY;
    "king_on_home_square", 2, "bits", CLASSIC_ONLY;
    "king_castled_zone", 4, "bits", CLASSIC_ONLY;
    "pawn_shield", 24, "one-hot", ANY;
    "king_file_openness", 12, "bits", ANY;
    "pawn_storm", 24, "one-hot", ANY;
    "ring_attackers", 2, "count/6", ANY;
    "ring_attack_weight", 2, "count/16", ANY;
    "ring_defended", 2, "count/8", ANY;
    "ring_holes", 2, "count/8", ANY;
    "king_escape_squares", 2, "count/8", ANY;
    "back_rank_risk", 2, "bits", ANY;
    "king_distance", 6, "one-hot", ANY;
    "king_tropism", 2, "count/8", ANY;
    "virtual_mobility", 2, "count/27", ANY;
};

static MOBILITY: [FeatureSpec; 10] = feature_specs! {
    "mobility_ratio", 1, "ratio", ANY;
    "mobility_by_type", 10, "count/16", ANY;
    "safe_mobility_by_type", 10, "count/16", ANY;
    "mobility_diff_by_type", 5, "diff/16", ANY;
    "space", 2, "count/32", ANY;
    "controlled_squares", 3, "count/48", ANY;
    "centre_control", 2, "count/4", ANY;
    "extended_centre_control", 2, "count/16", ANY;
    "immobile_pieces", 2, "count/4", ANY;
    "total_mobility", 2, "count/96", ANY;
};

static ATTACKS: [FeatureSpec; 12] = feature_specs! {
    "attacked_square_count", 3, "count/48", ANY;
    "attacked_count", 2, "count/8", ANY;
    "attacked_value", 2, "count/20", ANY;
    "hanging_count", 2, "count/4", ANY;
    "hanging_value", 2, "count/20", ANY;
    "en_prise_count", 2, "count/4", ANY;
    "en_prise_value", 2, "count/20", ANY;
    "en_prise_max_value", 2, "count/9", ANY;
    "pinned_count", 2, "count/4", ANY;
    "pinned_value", 2, "count/20", ANY;
    "skewer_candidates", 2, "count/4", ANY;
    "defended_count", 2, "count/16", ANY;
};

static EXCHANGE: [FeatureSpec; 8] = feature_specs! {
    "us.see_best_capture", 1, "diff/9", ANY;
    "us.see_positive_capture_count", 1, "count/8", ANY;
    "us.see_equal_capture_count", 1, "count/8", ANY;
    "us.see_positive_total", 1, "count/20", ANY;
    "them.see_best_capture", 1, "diff/9", ANY;
    "them.see_positive_capture_count", 1, "count/8", ANY;
    "them.see_equal_capture_count", 1, "count/8", ANY;
    "them.see_positive_total", 1, "count/20", ANY;
};

static THREATS: [FeatureSpec; 0] = feature_specs! {};

static ENDGAME: [FeatureSpec; 7] = feature_specs! {
    "king_centralisation", 2, "count/3", ANY;
    "race_plies", 2, "count/8", ANY;
    "race_plies_diff", 1, "diff/8", ANY;
    "opposition", 3, "one-hot", ANY;
    "key_square_occupied", 2, "bits", ANY;
    "wrong_colour_bishop", 2, "bits", ANY;
    "drawish_material", 3, "one-hot", ANY;
};

static HISTORY: [FeatureSpec; 11] = feature_specs! {
    "halfmove_bucket", 8, "one-hot", ANY;
    "halfmove_known", 1, "bit", ANY;
    "repetition_seen", 1, "bit", ANY;
    "repetition_available_us", 1, "bit", ANY;
    "captures_in_last_8", 1, "count/8", ANY;
    "checks_in_last_8", 1, "count/8", ANY;
    "quiet_plies", 1, "count/16", ANY;
    "material_trend", 1, "diff/9", ANY;
    "last_move_victim", 5, "one-hot", ANY;
    "last_move_mover", 6, "one-hot", ANY;
    "history_known", 1, "bit", ANY;
};

static TACTICS: [FeatureSpec; 70] = feature_specs! {
    "us.check_available", 1, "bit", ANY;
    "us.check_count", 1, "count/8", ANY;
    "us.check_by_piece", 5, "bits", ANY;
    "us.safe_check_available", 1, "bit", ANY;
    "us.safe_check_count", 1, "count/8", ANY;
    "us.safe_check_by_piece", 5, "bits", ANY;
    "us.double_check_available", 1, "bit", ANY;
    "us.discovered_check_available", 1, "bit", ANY;
    "us.mate_in_1", 1, "bit", ANY;
    "us.stalemate_in_1", 1, "bit", ANY;
    "us.promotion_available", 1, "bit", ANY;
    "us.promotion_file", 8, "mask8", ANY;
    "us.promotion_piece", 4, "bits", ANY;
    "us.safe_promotion_available", 1, "bit", ANY;
    "us.safe_promotion_file", 8, "mask8", ANY;
    "us.capture_available", 1, "bit", ANY;
    "us.capture_count", 1, "count/16", ANY;
    "us.winning_capture_available", 1, "bit", ANY;
    "us.winning_capture_max_gain", 1, "count/9", ANY;
    "us.captures_hanging_available", 1, "bit", ANY;
    "us.hanging_victim_max_value", 1, "count/9", ANY;
    "us.equal_capture_count", 1, "count/8", ANY;
    "us.losing_capture_count", 1, "count/8", ANY;
    "us.fork_available", 1, "bit", ANY;
    "us.fork_count", 1, "count/4", ANY;
    "us.fork_max_value", 1, "count/9", ANY;
    "us.knight_fork_available", 1, "bit", ANY;
    "us.royal_fork_available", 1, "bit", ANY;
    "us.pin_creation_available", 1, "bit", ANY;
    "us.pin_creation_count", 1, "count/4", ANY;
    "us.skewer_creation_available", 1, "bit", ANY;
    "us.discovered_attack_available", 1, "bit", ANY;
    "us.legal_move_count", 1, "count/64", ANY;
    "us.only_moves", 1, "bit", ANY;
    "us.facts_available", 1, "bit", ANY;
    "them.check_available", 1, "bit", ANY;
    "them.check_count", 1, "count/8", ANY;
    "them.check_by_piece", 5, "bits", ANY;
    "them.safe_check_available", 1, "bit", ANY;
    "them.safe_check_count", 1, "count/8", ANY;
    "them.safe_check_by_piece", 5, "bits", ANY;
    "them.double_check_available", 1, "bit", ANY;
    "them.discovered_check_available", 1, "bit", ANY;
    "them.mate_in_1", 1, "bit", ANY;
    "them.stalemate_in_1", 1, "bit", ANY;
    "them.promotion_available", 1, "bit", ANY;
    "them.promotion_file", 8, "mask8", ANY;
    "them.promotion_piece", 4, "bits", ANY;
    "them.safe_promotion_available", 1, "bit", ANY;
    "them.safe_promotion_file", 8, "mask8", ANY;
    "them.capture_available", 1, "bit", ANY;
    "them.capture_count", 1, "count/16", ANY;
    "them.winning_capture_available", 1, "bit", ANY;
    "them.winning_capture_max_gain", 1, "count/9", ANY;
    "them.captures_hanging_available", 1, "bit", ANY;
    "them.hanging_victim_max_value", 1, "count/9", ANY;
    "them.equal_capture_count", 1, "count/8", ANY;
    "them.losing_capture_count", 1, "count/8", ANY;
    "them.fork_available", 1, "bit", ANY;
    "them.fork_count", 1, "count/4", ANY;
    "them.fork_max_value", 1, "count/9", ANY;
    "them.knight_fork_available", 1, "bit", ANY;
    "them.royal_fork_available", 1, "bit", ANY;
    "them.pin_creation_available", 1, "bit", ANY;
    "them.pin_creation_count", 1, "count/4", ANY;
    "them.skewer_creation_available", 1, "bit", ANY;
    "them.discovered_attack_available", 1, "bit", ANY;
    "them.legal_move_count", 1, "count/64", ANY;
    "them.only_moves", 1, "bit", ANY;
    "them.facts_available", 1, "bit", ANY;
};

static PLANES: [FeatureSpec; 8] = feature_specs! {
    "attacked_by_us", 64, "plane", ANY;
    "attacked_by_them", 64, "plane", ANY;
    "attacked_by_our_pawns", 64, "plane", ANY;
    "attacked_by_their_pawns", 64, "plane", ANY;
    "our_hanging", 64, "plane", ANY;
    "their_hanging", 64, "plane", ANY;
    "our_pinned", 64, "plane", ANY;
    "their_pinned", 64, "plane", ANY;
};

static V1_GROUPS: [GroupSpec; 14] = [
    GroupSpec {
        name: "placement",
        version: 1,
        width: 768,
        features: &PLACEMENT,
    },
    GroupSpec {
        name: "state",
        version: 2,
        width: 16,
        features: &STATE,
    },
    GroupSpec {
        name: "material",
        version: 1,
        width: 26,
        features: &MATERIAL,
    },
    GroupSpec {
        name: "pawns",
        version: 1,
        width: 165,
        features: &PAWNS,
    },
    GroupSpec {
        name: "pieces",
        version: 2,
        width: 35,
        features: &PIECES,
    },
    GroupSpec {
        name: "king",
        version: 3,
        width: 120,
        features: &KING,
    },
    GroupSpec {
        name: "mobility",
        version: 1,
        width: 39,
        features: &MOBILITY,
    },
    GroupSpec {
        name: "attacks",
        version: 2,
        width: 25,
        features: &ATTACKS,
    },
    GroupSpec {
        name: "exchange",
        version: 1,
        width: 8,
        features: &EXCHANGE,
    },
    GroupSpec {
        name: "threats",
        version: 1,
        width: 0,
        features: &THREATS,
    },
    GroupSpec {
        name: "tactics",
        version: 2,
        width: 120,
        features: &TACTICS,
    },
    GroupSpec {
        name: "endgame",
        version: 2,
        width: 15,
        features: &ENDGAME,
    },
    GroupSpec {
        name: "history",
        version: 2,
        width: 27,
        features: &HISTORY,
    },
    GroupSpec {
        name: "planes",
        version: 1,
        width: 512,
        features: &PLANES,
    },
];

static V1: Schema = Schema {
    semver: "1.0.0",
    groups: &V1_GROUPS,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_widths_are_the_sum_of_their_features() {
        for group in Schema::v1().groups() {
            let sum: usize = group.features.iter().map(|f| f.width).sum();
            assert_eq!(sum, group.width, "group {}", group.name);
            for (index, feature) in group.features.iter().enumerate() {
                let expected: usize = group.features[..index].iter().map(|f| f.width).sum();
                assert_eq!(feature.offset, expected, "{}.{}", group.name, feature.name);
            }
        }
    }

    #[test]
    fn the_schema_is_as_wide_as_features_md_says() {
        assert_eq!(Schema::v1().width(), 1876);
        assert!(Schema::v1().feature_count() <= MAX_FEATURES);
    }
}
