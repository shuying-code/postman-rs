//! Variable scopes and resolution, including Postman-style dynamic variables.
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;

/// Ordered variable scopes from highest to lowest priority.
#[derive(Debug, Clone, Default)]
pub struct VarContext {
    /// Each map is a scope; earlier maps win.
    pub scopes: Vec<HashMap<String, String>>,
}

impl VarContext {
    pub fn new() -> Self {
        VarContext { scopes: Vec::new() }
    }

    /// Push a scope at the lowest priority.
    pub fn push_low(&mut self, map: HashMap<String, String>) {
        self.scopes.push(map);
    }

    /// Push a scope at the highest priority.
    pub fn push_high(&mut self, map: HashMap<String, String>) {
        self.scopes.insert(0, map);
    }

    pub fn set_local(&mut self, k: &str, v: &str) {
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }
        self.scopes[0].insert(k.to_string(), v.to_string());
    }

    pub fn lookup(&self, key: &str) -> Option<String> {
        for scope in &self.scopes {
            if let Some(v) = scope.get(key) {
                return Some(v.clone());
            }
        }
        None
    }

    /// Flatten all scopes into a single map (higher-priority scopes win).
    /// Used by the script engine so `pm.variables.get(...)` sees resolved values.
    pub fn merged(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        for scope in self.scopes.iter().rev() {
            for (k, v) in scope {
                m.insert(k.clone(), v.clone());
            }
        }
        m
    }

    /// Resolve `{{var}}` occurrences recursively (bounded depth).
    pub fn resolve(&self, input: &str) -> String {
        resolve_str(input, self, 0, 32)
    }
}

fn resolve_str(input: &str, ctx: &VarContext, depth: usize, max: usize) -> String {
    if depth >= max {
        return input.to_string();
    }
    let re = Regex::new(r"\{\{\s*([^}]+?)\s*\}\}").unwrap();
    if !re.is_match(input) {
        return input.to_string();
    }
    let replaced = re
        .replace_all(input, |caps: &regex::Captures| {
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if let Some(dv) = dynamic_value(name) {
                return dv;
            }
            match ctx.lookup(name) {
                Some(v) => resolve_str(&v, ctx, depth + 1, max),
                None => caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default(),
            }
        })
        .to_string();
    replaced
}

/// Postman built-in dynamic variables (`{{$name}}`).
pub fn dynamic_value(name: &str) -> Option<String> {
    let mut rng = rand::thread_rng();
    let first = [
        "James", "Mary", "John", "Patricia", "Robert", "Jennifer", "Michael", "Linda", "David",
        "Elizabeth", "William", "Barbara", "Richard", "Susan", "Joseph", "Jessica",
    ];
    let last = [
        "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez",
        "Martinez", "Hernandez", "Lopez", "Wilson", "Anderson", "Thomas", "Taylor",
    ];
    let cities = [
        "Beijing", "Shanghai", "Tokyo", "London", "Paris", "Berlin", "Sydney", "Toronto", "New York",
        "Singapore",
    ];
    let countries = [
        "China", "Japan", "United States", "Germany", "France", "Australia", "Canada", "Singapore",
        "Brazil", "India",
    ];
    let colors = ["red", "green", "blue", "yellow", "purple", "orange", "black", "white"];
    let lorem_words = [
        "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit", "sed",
        "eiusmod", "tempor", "incididunt", "ut", "labore", "et", "dolore", "magna", "aliqua",
        "enim", "minim", "veniam", "quis", "nostrud", "exercitation", "ullamco",
    ];
    let mut pick = |arr: &[&str]| arr[rng.gen_range(0..arr.len())].to_string();
    let v = match name {
        "$randomInt" => rng.gen_range(0..1001).to_string(),
        "$randomUUID" | "$guid" => uuid::Uuid::new_v4().to_string(),
        "$timestamp" => chrono::Utc::now().timestamp().to_string(),
        "$isoTimestamp" => chrono::Utc::now().to_rfc3339(),
        "$randomAlphaNumeric" => {
            const CH: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            (0..16)
                .map(|_| CH[rng.gen_range(0..CH.len())] as char)
                .collect()
        }
        "$randomBoolean" => {
            if rng.gen_bool(0.5) {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        "$randomFirstName" => pick(&first),
        "$randomLastName" => pick(&last),
        "$randomFullName" => format!("{} {}", pick(&first), pick(&last)),
        "$randomEmail" => format!(
            "{}.{}@example.com",
            pick(&first).to_lowercase(),
            pick(&last).to_lowercase()
        ),
        "$randomPassword" => {
            const CH: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#%";
            (0..12)
                .map(|_| CH[rng.gen_range(0..CH.len())] as char)
                .collect()
        }
        "$randomCity" => pick(&cities),
        "$randomCountry" => pick(&countries),
        "$randomPhoneNumber" => format!(
            "{}{}{}",
            rng.gen_range(100..1000),
            rng.gen_range(100..1000),
            rng.gen_range(1000..10000)
        ),
        "$randomIP" => format!(
            "{}.{}.{}.{}",
            rng.gen_range(1..255),
            rng.gen_range(0..255),
            rng.gen_range(0..255),
            rng.gen_range(1..255)
        ),
        "$randomColor" => pick(&colors),
        "$randomHexColor" => format!(
            "#{:02x}{:02x}{:02x}",
            rng.gen_range(0..256),
            rng.gen_range(0..256),
            rng.gen_range(0..256)
        ),
        "$randomAbbreviation" => {
            const CH: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
            (0..3)
                .map(|_| CH[rng.gen_range(0..CH.len())] as char)
                .collect()
        }
        "$randomNumeric" => {
            const CH: &[u8] = b"0123456789";
            (0..8).map(|_| CH[rng.gen_range(0..CH.len())] as char).collect()
        }
        "$randomHexadecimal" => {
            const CH: &[u8] = b"0123456789abcdef";
            (0..16).map(|_| CH[rng.gen_range(0..CH.len())] as char).collect()
        }
        "$randomStreetAddress" => format!(
            "{} {} St",
            pick(&["1", "12", "123", "456", "1789", "9021"]),
            pick(&last)
        ),
        "$randomJobTitle" => format!(
            "{} {}",
            pick(&["Lead", "Senior", "Junior", "Principal", "Staff"]),
            pick(&["Engineer", "Developer", "Analyst", "Designer", "Manager"])
        ),
        "$randomJobDescriptor" => pick(&["Dynamic", "Global", "Product", "Internal", "Regional"]).to_string(),
        "$randomJobArea" => pick(&["Operations", "Engineering", "Marketing", "Security", "Data"]).to_string(),
        "$randomJobType" => pick(&["Manager", "Coordinator", "Specialist", "Executive", "Director"]).to_string(),
        "$randomCompanyName" => format!("{} {}", pick(&last), pick(&["Inc", "LLC", "Group", "Labs", "Systems"])),
        "$randomCompanySuffix" => pick(&["Inc", "LLC", "Group", "Labs", "Systems"]).to_string(),
        "$randomBs" => format!(
            "{} {} {}",
            pick(&["synergize", "leverage", "optimize", "enable", "aggregate"]),
            pick(&["scalable", "cross-platform", "distributed", "real-time", "open-source"]),
            pick(&["models", "platforms", "solutions", "markets", "paradigms"])
        ),
        "$randomBsAdjective" => pick(&["scalable", "cutting-edge", "innovative", "next-generation", "robust"]).to_string(),
        "$randomBsBuzz" => pick(&["synergize", "leverage", "optimize", "aggregate", "orchestrate"]).to_string(),
        "$randomBsNoun" => pick(&["models", "platforms", "solutions", "markets", "paradigms"]).to_string(),
        "$randomCatchPhrase" => format!(
            "{} {} {}",
            pick(&["Adaptive", "Seamless", "Integrated", "Robust", "Scalable"]),
            pick(&["web-enabled", "mission-critical", "next-generation", "cross-platform"]),
            pick(&["technology", "architecture", "framework", "interface"])
        ),
        "$randomLatitude" => format!("{:.6}", rng.gen_range(-90.0f64..90.0)),
        "$randomLongitude" => format!("{:.6}", rng.gen_range(-180.0f64..180.0)),
        "$randomWord" => pick(&["alpha", "beta", "gamma", "delta", "omega", "lorem", "ipsum"]).to_string(),
        "$randomWords" => format!(
            "{} {} {}",
            pick(&["alpha", "beta", "gamma"]),
            pick(&["delta", "omega", "lorem"]),
            pick(&["ipsum", "dolor", "sit"])
        ),
        "$randomCurrencyCode" => pick(&["USD", "EUR", "CNY", "JPY", "GBP"]).to_string(),
        "$randomCurrencyName" => pick(&["US Dollar", "Euro", "Yuan", "Yen", "Pound"]).to_string(),
        "$randomCurrencySymbol" => pick(&["$", "€", "¥", "£"]).to_string(),
        "$randomPrice" => format!("{}.{:02}", rng.gen_range(1..1000), rng.gen_range(0..100)),
        "$randomProduct" => pick(&["Chair", "Table", "Keyboard", "Mouse", "Monitor"]).to_string(),
        "$randomProductAdjective" => pick(&["Ergonomic", "Sleek", "Durable", "Compact", "Premium"]).to_string(),
        "$randomProductMaterial" => pick(&["Wood", "Steel", "Plastic", "Aluminum", "Leather"]).to_string(),
        "$randomProductName" => pick(&["Chair", "Lamp", "Stand", "Dock", "Hub"]).to_string(),
        "$randomDepartment" => pick(&["Books", "Electronics", "Garden", "Tools", "Sports"]).to_string(),
        "$randomFileName" => format!("{}.{}", pick(&["report", "data", "image", "backup", "log"]), pick(&["txt", "json", "png", "csv", "pdf"])),
        "$randomCommonFileExt" => pick(&["txt", "json", "png", "csv", "pdf"]).to_string(),
        "$randomFileExt" => pick(&["dat", "bin", "tmp", "bak", "xml"]).to_string(),
        "$randomFileType" => pick(&["application/json", "text/plain", "image/png", "application/pdf"]).to_string(),
        "$randomDirectoryPath" => format!("/{}/{}", pick(&["home", "var", "usr", "opt"]), pick(&["data", "logs", "tmp", "app"])),
        "$randomFilePath" => format!("/{}/{}/{}.{}", pick(&["home", "var", "opt"]), pick(&["data", "logs", "app"]), pick(&["file", "report", "backup"]), pick(&["txt", "json", "log"])),
        "$randomUserAgent" => pick(&[
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_0) AppleWebKit/605.1.15",
            "Mozilla/5.0 (X11; Linux x86_64) Gecko/20100101 Firefox/118.0",
        ]).to_string(),
        "$randomProtocol" => pick(&["http", "https"]).to_string(),
        "$randomDomainName" => format!("{}.{}", pick(&["example", "acme", "test", "service"]), pick(&["com", "net", "org", "io"])),
        "$randomDomainSuffix" => pick(&["com", "net", "org", "io"]).to_string(),
        "$randomUrl" => format!("https://{}.{}/{}", pick(&["example", "acme", "test"]), pick(&["com", "io", "net"]), pick(&["api", "v1", "docs"])),
        "$randomIPV6" => format!(
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            rng.gen_range(0..65535), rng.gen_range(0..65535), rng.gen_range(0..65535), rng.gen_range(0..65535),
            rng.gen_range(0..65535), rng.gen_range(0..65535), rng.gen_range(0..65535), rng.gen_range(0..65535)
        ),
        "$randomMACAddress" => format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            rng.gen_range(0..256), rng.gen_range(0..256), rng.gen_range(0..256),
            rng.gen_range(0..256), rng.gen_range(0..256), rng.gen_range(0..256)
        ),
        "$randomLocale" => pick(&["zh_CN", "en_US", "ja_JP", "de_DE", "fr_FR"]).to_string(),
        "$randomSemver" => format!("{}.{}.{}", rng.gen_range(0..10), rng.gen_range(0..20), rng.gen_range(0..50)),
        "$randomBankAccount" => format!("{:08}", rng.gen_range(0..100_000_000)),
        "$randomCreditCardMask" => format!("**** **** **** {:04}", rng.gen_range(0..10000)),
        "$randomDateFuture" => (chrono::Utc::now() + chrono::Duration::days(rng.gen_range(1..365))).to_rfc3339(),
        "$randomDatePast" => (chrono::Utc::now() - chrono::Duration::days(rng.gen_range(1..365))).to_rfc3339(),
        "$randomDateRecent" => (chrono::Utc::now() - chrono::Duration::hours(rng.gen_range(1..72))).to_rfc3339(),
        "$randomLoremWord" => pick(&lorem_words),
        "$randomLoremWords" => (0..5)
            .map(|_| pick(&lorem_words))
            .collect::<Vec<_>>()
            .join(" "),
        "$randomLoremSentence" => format!(
            "{}.",
            (0..8).map(|_| pick(&lorem_words)).collect::<Vec<_>>().join(" ")
        ),
        "$randomLoremSentences" => (0..3)
            .map(|_| {
                format!(
                    "{}.",
                    (0..8).map(|_| pick(&lorem_words)).collect::<Vec<_>>().join(" ")
                )
            })
            .collect::<Vec<_>>()
            .join(" "),
        "$randomLoremParagraph" => (0..4)
            .map(|_| {
                format!(
                    "{}.",
                    (0..10).map(|_| pick(&lorem_words)).collect::<Vec<_>>().join(" ")
                )
            })
            .collect::<Vec<_>>()
            .join(" "),
        "$randomLoremParagraphs" => (0..3)
            .map(|_| {
                (0..4)
                    .map(|_| {
                        format!(
                            "{}.",
                            (0..10).map(|_| pick(&lorem_words)).collect::<Vec<_>>().join(" ")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
        "$randomLoremText" => (0..5)
            .map(|_| {
                format!(
                    "{}.",
                    (0..9).map(|_| pick(&lorem_words)).collect::<Vec<_>>().join(" ")
                )
            })
            .collect::<Vec<_>>()
            .join(" "),
        "$randomLoremSlug" => (0..4)
            .map(|_| pick(&lorem_words))
            .collect::<Vec<_>>()
            .join("-"),
        "$randomImageUrl" | "$randomAbstractImage" | "$randomAnimalsImage" | "$randomBusinessImage"
        | "$randomCatsImage" | "$randomCityImage" | "$randomFoodImage" | "$randomNightlifeImage"
        | "$randomFashionImage" | "$randomPeopleImage" | "$randomNatureImage" | "$randomSportsImage"
        | "$randomTransportImage" => format!(
            "https://loremflickr.com/320/240/{}",
            pick(&["abstract", "animals", "business", "cats", "city", "food"])
        ),
        "$randomAvatar" => format!(
            "https://i.pravatar.cc/150?u={}",
            uuid::Uuid::new_v4().simple()
        ),
        "$randomUserName" => format!(
            "{}.{}{}",
            pick(&first).to_lowercase(),
            pick(&last).to_lowercase(),
            rng.gen_range(1..99)
        ),
        "$randomBankAccountName" => pick(&first),
        "$randomBankAccountIban" | "$randomIban" => format!(
            "{}{}{}",
            pick(&["DE", "GB", "FR", "CN"]),
            rng.gen_range(10..99),
            (0..16)
                .map(|_| rng.gen_range(0..10).to_string())
                .collect::<Vec<_>>()
                .join("")
        ),
        "$randomBankAccountBic" | "$randomBic" => format!(
            "{}{}XXXX",
            pick(&["DEUT", "BNPA", "CITI", "HSBC"]),
            pick(&["DE", "GB", "FR", "CN"])
        ),
        "$randomBankAccountNumber" => format!("{:08}", rng.gen_range(0..100_000_000)),
        "$randomBankCountry" => pick(&countries),
        "$randomCreditCardNumber" => (0..16)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect::<Vec<_>>()
            .join(""),
        "$randomAmount" => format!("{}.{:02}", rng.gen_range(0..10000), rng.gen_range(0..100)),
        _ => return None,
    };
    Some(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> VarContext {
        let mut c = VarContext::new();
        let mut globals = HashMap::new();
        globals.insert("host".to_string(), "https://api.test".to_string());
        let mut env = HashMap::new();
        env.insert("host".to_string(), "https://env.test".to_string());
        env.insert("path".to_string(), "users".to_string());
        // scopes are ordered high -> low; environment must beat globals
        c.scopes.push(env);
        c.scopes.push(globals);
        c
    }

    #[test]
    fn resolves_and_prioritizes() {
        let c = ctx();
        assert_eq!(c.resolve("{{host}}/{{path}}"), "https://env.test/users");
    }

    #[test]
    fn recursive_resolution() {
        let mut c = VarContext::new();
        let mut env = HashMap::new();
        env.insert("a".to_string(), "{{b}}".to_string());
        env.insert("b".to_string(), "final".to_string());
        c.push_low(env);
        assert_eq!(c.resolve("{{a}}"), "final");
    }

    #[test]
    fn dynamic_vars() {
        assert!(dynamic_value("$randomInt").unwrap().parse::<i32>().is_ok());
        assert_eq!(dynamic_value("$randomBoolean").unwrap().len() >= 4, true);
        assert!(dynamic_value("$nonexistent").is_none());
    }

    #[test]
    fn leaves_unknown_intact() {
        let c = ctx();
        assert_eq!(c.resolve("{{missing}}"), "{{missing}}");
    }

    #[test]
    fn lorem_and_image_dynamic_vars() {
        assert!(dynamic_value("$randomLoremSentence").unwrap().ends_with('.'));
        assert!(dynamic_value("$randomImageUrl").unwrap().starts_with("https://"));
        assert_eq!(dynamic_value("$randomCreditCardNumber").unwrap().len(), 16);
        assert!(dynamic_value("$randomLoremSlug").unwrap().contains('-'));
        assert!(!dynamic_value("$randomAvatar").unwrap().is_empty());
    }
}
