#[cfg(test)]
extern crate num_traits;
extern crate geo_types;

pub use geo_types::Coordinate;

static BASE32_CODES: &'static [char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'b',
                                         'c', 'd', 'e', 'f', 'g', 'h', 'j', 'k', 'm', 'n', 'p',
                                         'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];
#[derive(Debug, Clone, PartialEq)]
pub struct Neighbors {
    pub sw: String,
    pub s: String,
    pub se: String,
    pub w: String,
    pub e: String,
    pub nw: String,
    pub n: String,
    pub ne: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// North
    N,
    /// North-east
    Ne,
    /// Eeast
    E,
    /// South-east
    Se,
    /// South
    S,
    /// South-west
    Sw,
    /// West
    W,
    /// North-west
    Nw,
}

impl Direction {
    fn to_tuple(self) -> (i8, i8) {
        match self {
            Direction::Sw => (-1, -1),
            Direction::S => (-1, 0),
            Direction::Se => (-1, 1),
            Direction::W => (0, -1),
            Direction::E => (0, 1),
            Direction::Nw => (1, -1),
            Direction::N => (1, 0),
            Direction::Ne => (1, 1),
        }
    }
}

/// Encode a coordinate to a geohash with length `len`.
///
/// # Examples
///
/// Encoding a coordinate to a length five geohash:
///
/// ```rust
/// let coord = geohash::Coordinate { x: -120.6623, y: 35.3003 };
///
/// let geohash_string = geohash::encode(coord, 5);
///
/// assert_eq!(geohash_string, "9q60y");
/// ```
///
/// Encoding a coordinate to a length ten geohash:
///
/// ```rust
/// let coord = geohash::Coordinate { x: -120.6623, y: 35.3003 };
///
/// let geohash_string = geohash::encode(coord, 10);
///
/// assert_eq!(geohash_string, "9q60y60rhs");
/// ```
pub fn encode(c: Coordinate<f64>, len: usize) -> String {
    let mut out = String::with_capacity(len);

    let mut bits: i8 = 0;
    let mut bits_total: i8 = 0;
    let mut hash_value: usize = 0;
    let mut max_lat = 90f64;
    let mut min_lat = -90f64;
    let mut max_lon = 180f64;
    let mut min_lon = -180f64;
    let mut mid: f64;

    while out.len() < len {
        if bits_total % 2 == 0 {
            mid = (max_lon + min_lon) / 2f64;
            if c.x > mid {
                hash_value = (hash_value << 1) + 1usize;
                min_lon = mid;
            } else {
                hash_value <<= 1;
                max_lon = mid;
            }
        } else {
            mid = (max_lat + min_lat) / 2f64;
            if c.y > mid {
                hash_value = (hash_value << 1) + 1usize;
                min_lat = mid;
            } else {
                hash_value <<= 1;
                max_lat = mid;
            }
        }

        bits += 1;
        bits_total += 1;

        if bits == 5 {
            let code: char = BASE32_CODES[hash_value];
            out.push(code);
            bits = 0;
            hash_value = 0;
        }
    }
    out
}

/// ### Decode geohash string into latitude, longitude
///
/// Parameters:
/// Geohash encoded `&str`
///
/// Returns:
/// A four-element tuple describs a bound box:
/// * min_lat
/// * max_lat
/// * min_lon
/// * max_lon
pub fn decode_bbox(hash_str: &str) -> (Coordinate<f64>, Coordinate<f64>) {
    let mut is_lon = true;
    let mut max_lat = 90f64;
    let mut min_lat = -90f64;
    let mut max_lon = 180f64;
    let mut min_lon = -180f64;
    let mut mid: f64;
    let mut hash_value: usize;

    for c in hash_str.chars() {
        hash_value = BASE32_CODES.iter().position(|n| *n == c).unwrap();

        for bs in 0..5 {
            let bit = (hash_value >> (4 - bs)) & 1usize;
            if is_lon {
                mid = (max_lon + min_lon) / 2f64;

                if bit == 1 {
                    min_lon = mid;
                } else {
                    max_lon = mid;
                }
            } else {
                mid = (max_lat + min_lat) / 2f64;

                if bit == 1 {
                    min_lat = mid;
                } else {
                    max_lat = mid;
                }
            }
            is_lon = !is_lon;
        }
    }

    (Coordinate {
         x: min_lon,
         y: min_lat,
     },
     Coordinate {
         x: max_lon,
         y: max_lat,
     })
}

/// Decode a geohash into a coordinate with some longitude/latitude error. The
/// return value is `(<coordinate>, <longitude error>, <latitude error>)`.
///
/// # Examples
///
/// Decoding a length five geohash:
///
/// ```rust
/// let geohash_str = "9q60y";
///
/// let decoded = geohash::decode(geohash_str);
///
/// assert_eq!(
///     decoded,
///     (
///         geohash::Coordinate {
///             x: -120.65185546875,
///             y: 35.31005859375,
///         },
///         0.02197265625,
///         0.02197265625,
///     ),
/// );
/// ```
///
/// Decoding a length ten geohash:
///
/// ```rust
/// let geohash_str = "9q60y60rhs";
///
/// let decoded = geohash::decode(geohash_str);
///
/// assert_eq!(
///     decoded,
///     (
///         geohash::Coordinate {
///             x: -120.66229999065399,
///             y: 35.300298035144806,
///         },
///         0.000002682209014892578,
///         0.000005364418029785156,
///     ),
/// );
/// ```
pub fn decode(hash_str: &str) -> (Coordinate<f64>, f64, f64) {
    let (c0, c1) = decode_bbox(hash_str);
    (Coordinate {
         x: (c0.x + c1.x) / 2f64,
         y: (c0.y + c1.y) / 2f64,
     },
     (c1.y - c0.y) / 2f64,
     (c1.x - c0.x) / 2f64)
}

pub fn neighbor(hash_str: &str, direction: Direction) -> String {
    let b = decode(hash_str);
    let c = b.0;
    let gl = match direction.to_tuple() {
        (dlat, dlng) => {
            Coordinate {
                x: c.x + 2f64 * b.1.abs() * (dlng as f64),
                y: c.y + 2f64 * b.2.abs() * (dlat as f64),
            }
        }
    };
    encode(gl, hash_str.len())
}

/// Find all neighboring geohashes for the given geohash.
///
/// # Examples
///
/// ```
/// let geohash_str = "9q60y60rhs";
///
/// let neighbors = geohash::neighbors(geohash_str);
///
/// assert_eq!(
///     neighbors,
///     geohash::Neighbors {
///         n: "9q60y60rhw".to_owned(),
///         ne: "9q60y60rhw".to_owned(),
///         e: "9q60y60rhs".to_owned(),
///         se: "9q60y60rhd".to_owned(),
///         s: "9q60y60rhd".to_owned(),
///         sw: "9q60y60rh6".to_owned(),
///         w: "9q60y60rhk".to_owned(),
///         nw: "9q60y60rhq".to_owned(),
///     }
/// );
/// ```
pub fn neighbors(hash_str: &str) -> Neighbors {
    Neighbors {
        sw: neighbor(hash_str, Direction::Sw),
        s: neighbor(hash_str, Direction::S),
        se: neighbor(hash_str, Direction::Se),
        w: neighbor(hash_str, Direction::W),
        e: neighbor(hash_str, Direction::E),
        nw: neighbor(hash_str, Direction::Nw),
        n: neighbor(hash_str, Direction::N),
        ne: neighbor(hash_str, Direction::Ne),
    }
}

#[cfg(test)]
mod test {
    use {encode, decode, neighbors};
    use geo_types::Coordinate;
    use num_traits::Float;

    #[test]
    fn test_encode() {
        let c0 = Coordinate {
            x: 112.5584f64,
            y: 37.8324f64,
        };
        assert_eq!(encode(c0, 9usize), "ww8p1r4t8".to_string());
        let c1 = Coordinate {
            x: 117f64,
            y: 32f64,
        };
        assert_eq!(encode(c1, 3usize), "wte".to_string());
    }

    #[test]
    fn test_decode() {
        let (c, _, _) = decode("ww8p1r4t8");
        assert_eq!(Float::abs_sub(c.y, 37.8324f64) < 1e-4f64, true);
        assert_eq!(Float::abs_sub(c.x, 112.5584f64) < 1e-4f64, true);
    }


    #[test]
    fn test_neighbor() {
        let ns = neighbors("ww8p1r4t8");
        assert_eq!(ns.sw, "ww8p1r4mr");
        assert_eq!(ns.s, "ww8p1r4t2");
        assert_eq!(ns.se, "ww8p1r4t3");
        assert_eq!(ns.w, "ww8p1r4mx");
        assert_eq!(ns.e, "ww8p1r4t9");
        assert_eq!(ns.nw, "ww8p1r4mz");
        assert_eq!(ns.n, "ww8p1r4tb");
        assert_eq!(ns.ne, "ww8p1r4tc");
    }
}
