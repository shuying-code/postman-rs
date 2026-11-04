//! Variable scopes and resolution, including Postman-style dynamic variables.
use rand::Rng; // 引入依赖
use regex::Regex; // 引入依赖
use std::collections::HashMap; // 引入依赖

/// Ordered variable scopes from highest to lowest priority.
#[derive(Debug, Clone, Default)] // 语句
pub struct VarContext { // 结构体 VarContext
    /// Each map is a scope; earlier maps win.
    pub scopes: Vec<HashMap<String, String>>, // 字段 scopes
} // 块结束

impl VarContext { // 实现
    pub fn new() -> Self { // 函数 new
        VarContext { scopes: Vec::new() } // 语句
    } // 块结束

    /// Push a scope at the lowest priority.
    pub fn push_low(&mut self, map: HashMap<String, String>) { // 函数 push_low
        self.scopes.push(map); // 语句
    } // 块结束

    /// Push a scope at the highest priority.
    pub fn push_high(&mut self, map: HashMap<String, String>) { // 函数 push_high
        self.scopes.insert(0, map); // 语句
    } // 块结束

    pub fn set_local(&mut self, k: &str, v: &str) { // 函数 set_local
        if self.scopes.is_empty() { // 条件分支
            self.scopes.push(HashMap::new()); // 语句
        } // 块结束
        self.scopes[0].insert(k.to_string(), v.to_string()); // 语句
    } // 块结束

    pub fn lookup(&self, key: &str) -> Option<String> { // 函数 lookup
        for scope in &self.scopes { // 循环
            if let Some(v) = scope.get(key) { // 条件分支
                return Some(v.clone()); // 返回
            } // 块结束
        } // 块结束
        None // 语句
    } // 块结束

    /// Flatten all scopes into a single map (higher-priority scopes win).
    /// Used by the script engine so `pm.variables.get(...)` sees resolved values.
    pub fn merged(&self) -> HashMap<String, String> { // 函数 merged
        let mut m = HashMap::new(); // 变量 m
        for scope in self.scopes.iter().rev() { // 循环
            for (k, v) in scope { // 循环
                m.insert(k.clone(), v.clone()); // 语句
            } // 块结束
        } // 块结束
        m // 语句
    } // 块结束

    /// Resolve `{{var}}` occurrences recursively (bounded depth).
    pub fn resolve(&self, input: &str) -> String { // 函数 resolve
        resolve_str(input, self, 0, 32) // 调用 resolve_str
    } // 块结束
} // 块结束

fn resolve_str(input: &str, ctx: &VarContext, depth: usize, max: usize) -> String { // 函数 resolve_str
    if depth >= max { // 条件分支
        return input.to_string(); // 返回
    } // 块结束
    let re = Regex::new(r"\{\{\s*([^}]+?)\s*\}\}").unwrap(); // 变量 re
    if !re.is_match(input) { // 条件分支
        return input.to_string(); // 返回
    } // 块结束
    let replaced = re // 变量 replaced
        .replace_all(input, |caps: &regex::Captures| { // 语句
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or(""); // 变量 name
            if let Some(dv) = dynamic_value(name) { // 条件分支
                return dv; // 返回
            } // 块结束
            match ctx.lookup(name) { // 分支匹配
                Some(v) => resolve_str(&v, ctx, depth + 1, max), // 匹配分支
                None => caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default(), // 匹配分支
            } // 块结束
        }) // 语句
        .to_string(); // 语句
    replaced // 语句
} // 块结束

/// Postman built-in dynamic variables (`{{$name}}`).
pub fn dynamic_value(name: &str) -> Option<String> { // 函数 dynamic_value
    let mut rng = rand::thread_rng(); // 变量 rng
    let first = [ // 变量 first
        "James", "Mary", "John", "Patricia", "Robert", "Jennifer", "Michael", "Linda", "David", // 语句
        "Elizabeth", "William", "Barbara", "Richard", "Susan", "Joseph", "Jessica", // 语句
    ]; // 语句
    let last = [ // 变量 last
        "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", // 语句
        "Martinez", "Hernandez", "Lopez", "Wilson", "Anderson", "Thomas", "Taylor", // 语句
    ]; // 语句
    let cities = [ // 变量 cities
        "Beijing", "Shanghai", "Tokyo", "London", "Paris", "Berlin", "Sydney", "Toronto", "New York", // 语句
        "Singapore", // 语句
    ]; // 语句
    let countries = [ // 变量 countries
        "China", "Japan", "United States", "Germany", "France", "Australia", "Canada", "Singapore", // 语句
        "Brazil", "India", // 语句
    ]; // 语句
    let colors = ["red", "green", "blue", "yellow", "purple", "orange", "black", "white"]; // 变量 colors
    let lorem_words = [ // 变量 lorem_words
        "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit", "sed", // 语句
        "eiusmod", "tempor", "incididunt", "ut", "labore", "et", "dolore", "magna", "aliqua", // 语句
        "enim", "minim", "veniam", "quis", "nostrud", "exercitation", "ullamco", // 语句
    ]; // 语句
    let mut pick = |arr: &[&str]| arr[rng.gen_range(0..arr.len())].to_string(); // 变量 pick
    let v = match name { // 变量 v
        "$randomInt" => rng.gen_range(0..1001).to_string(), // 匹配分支
        "$randomUUID" | "$guid" => uuid::Uuid::new_v4().to_string(), // 匹配分支
        "$timestamp" => chrono::Utc::now().timestamp().to_string(), // 匹配分支
        "$isoTimestamp" => chrono::Utc::now().to_rfc3339(), // 匹配分支
        "$randomAlphaNumeric" => { // 匹配分支
            const CH: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"; // 常量 CH
            (0..16) // 语句
                .map(|_| CH[rng.gen_range(0..CH.len())] as char) // 语句
                .collect() // 语句
        } // 块结束
        "$randomBoolean" => { // 匹配分支
            if rng.gen_bool(0.5) { // 条件分支
                "true".to_string() // 语句
            } else { // 语句
                "false".to_string() // 语句
            } // 块结束
        } // 块结束
        "$randomFirstName" => pick(&first), // 匹配分支
        "$randomLastName" => pick(&last), // 匹配分支
        "$randomFullName" => format!("{} {}", pick(&first), pick(&last)), // 匹配分支
        "$randomEmail" => format!( // 匹配分支
            "{}.{}@example.com", // 语句
            pick(&first).to_lowercase(), // 调用 pick
            pick(&last).to_lowercase() // 调用 pick
        ), // 语句
        "$randomPassword" => { // 匹配分支
            const CH: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#%"; // 常量 CH
            (0..12) // 语句
                .map(|_| CH[rng.gen_range(0..CH.len())] as char) // 语句
                .collect() // 语句
        } // 块结束
        "$randomCity" => pick(&cities), // 匹配分支
        "$randomCountry" => pick(&countries), // 匹配分支
        "$randomPhoneNumber" => format!( // 匹配分支
            "{}{}{}", // 语句
            rng.gen_range(100..1000), // 语句
            rng.gen_range(100..1000), // 语句
            rng.gen_range(1000..10000) // 语句
        ), // 语句
        "$randomIP" => format!( // 匹配分支
            "{}.{}.{}.{}", // 语句
            rng.gen_range(1..255), // 语句
            rng.gen_range(0..255), // 语句
            rng.gen_range(0..255), // 语句
            rng.gen_range(1..255) // 语句
        ), // 语句
        "$randomColor" => pick(&colors), // 匹配分支
        "$randomHexColor" => format!( // 匹配分支
            "#{:02x}{:02x}{:02x}", // 语句
            rng.gen_range(0..256), // 语句
            rng.gen_range(0..256), // 语句
            rng.gen_range(0..256) // 语句
        ), // 语句
        "$randomAbbreviation" => { // 匹配分支
            const CH: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"; // 常量 CH
            (0..3) // 语句
                .map(|_| CH[rng.gen_range(0..CH.len())] as char) // 语句
                .collect() // 语句
        } // 块结束
        "$randomNumeric" => { // 匹配分支
            const CH: &[u8] = b"0123456789"; // 常量 CH
            (0..8).map(|_| CH[rng.gen_range(0..CH.len())] as char).collect() // 语句
        } // 块结束
        "$randomHexadecimal" => { // 匹配分支
            const CH: &[u8] = b"0123456789abcdef"; // 常量 CH
            (0..16).map(|_| CH[rng.gen_range(0..CH.len())] as char).collect() // 语句
        } // 块结束
        "$randomStreetAddress" => format!( // 匹配分支
            "{} {} St", // 语句
            pick(&["1", "12", "123", "456", "1789", "9021"]), // 调用 pick
            pick(&last) // 调用 pick
        ), // 语句
        "$randomJobTitle" => format!( // 匹配分支
            "{} {}", // 语句
            pick(&["Lead", "Senior", "Junior", "Principal", "Staff"]), // 调用 pick
            pick(&["Engineer", "Developer", "Analyst", "Designer", "Manager"]) // 调用 pick
        ), // 语句
        "$randomJobDescriptor" => pick(&["Dynamic", "Global", "Product", "Internal", "Regional"]).to_string(), // 匹配分支
        "$randomJobArea" => pick(&["Operations", "Engineering", "Marketing", "Security", "Data"]).to_string(), // 匹配分支
        "$randomJobType" => pick(&["Manager", "Coordinator", "Specialist", "Executive", "Director"]).to_string(), // 匹配分支
        "$randomCompanyName" => format!("{} {}", pick(&last), pick(&["Inc", "LLC", "Group", "Labs", "Systems"])), // 匹配分支
        "$randomCompanySuffix" => pick(&["Inc", "LLC", "Group", "Labs", "Systems"]).to_string(), // 匹配分支
        "$randomBs" => format!( // 匹配分支
            "{} {} {}", // 语句
            pick(&["synergize", "leverage", "optimize", "enable", "aggregate"]), // 调用 pick
            pick(&["scalable", "cross-platform", "distributed", "real-time", "open-source"]), // 调用 pick
            pick(&["models", "platforms", "solutions", "markets", "paradigms"]) // 调用 pick
        ), // 语句
        "$randomBsAdjective" => pick(&["scalable", "cutting-edge", "innovative", "next-generation", "robust"]).to_string(), // 匹配分支
        "$randomBsBuzz" => pick(&["synergize", "leverage", "optimize", "aggregate", "orchestrate"]).to_string(), // 匹配分支
        "$randomBsNoun" => pick(&["models", "platforms", "solutions", "markets", "paradigms"]).to_string(), // 匹配分支
        "$randomCatchPhrase" => format!( // 匹配分支
            "{} {} {}", // 语句
            pick(&["Adaptive", "Seamless", "Integrated", "Robust", "Scalable"]), // 调用 pick
            pick(&["web-enabled", "mission-critical", "next-generation", "cross-platform"]), // 调用 pick
            pick(&["technology", "architecture", "framework", "interface"]) // 调用 pick
        ), // 语句
        "$randomLatitude" => format!("{:.6}", rng.gen_range(-90.0f64..90.0)), // 匹配分支
        "$randomLongitude" => format!("{:.6}", rng.gen_range(-180.0f64..180.0)), // 匹配分支
        "$randomWord" => pick(&["alpha", "beta", "gamma", "delta", "omega", "lorem", "ipsum"]).to_string(), // 匹配分支
        "$randomWords" => format!( // 匹配分支
            "{} {} {}", // 语句
            pick(&["alpha", "beta", "gamma"]), // 调用 pick
            pick(&["delta", "omega", "lorem"]), // 调用 pick
            pick(&["ipsum", "dolor", "sit"]) // 调用 pick
        ), // 语句
        "$randomCurrencyCode" => pick(&["USD", "EUR", "CNY", "JPY", "GBP"]).to_string(), // 匹配分支
        "$randomCurrencyName" => pick(&["US Dollar", "Euro", "Yuan", "Yen", "Pound"]).to_string(), // 匹配分支
        "$randomCurrencySymbol" => pick(&["$", "€", "¥", "£"]).to_string(), // 匹配分支
        "$randomPrice" => format!("{}.{:02}", rng.gen_range(1..1000), rng.gen_range(0..100)), // 匹配分支
        "$randomProduct" => pick(&["Chair", "Table", "Keyboard", "Mouse", "Monitor"]).to_string(), // 匹配分支
        "$randomProductAdjective" => pick(&["Ergonomic", "Sleek", "Durable", "Compact", "Premium"]).to_string(), // 匹配分支
        "$randomProductMaterial" => pick(&["Wood", "Steel", "Plastic", "Aluminum", "Leather"]).to_string(), // 匹配分支
        "$randomProductName" => pick(&["Chair", "Lamp", "Stand", "Dock", "Hub"]).to_string(), // 匹配分支
        "$randomDepartment" => pick(&["Books", "Electronics", "Garden", "Tools", "Sports"]).to_string(), // 匹配分支
        "$randomFileName" => format!("{}.{}", pick(&["report", "data", "image", "backup", "log"]), pick(&["txt", "json", "png", "csv", "pdf"])), // 匹配分支
        "$randomCommonFileExt" => pick(&["txt", "json", "png", "csv", "pdf"]).to_string(), // 匹配分支
        "$randomFileExt" => pick(&["dat", "bin", "tmp", "bak", "xml"]).to_string(), // 匹配分支
        "$randomFileType" => pick(&["application/json", "text/plain", "image/png", "application/pdf"]).to_string(), // 匹配分支
        "$randomDirectoryPath" => format!("/{}/{}", pick(&["home", "var", "usr", "opt"]), pick(&["data", "logs", "tmp", "app"])), // 匹配分支
        "$randomFilePath" => format!("/{}/{}/{}.{}", pick(&["home", "var", "opt"]), pick(&["data", "logs", "app"]), pick(&["file", "report", "backup"]), pick(&["txt", "json", "log"])), // 匹配分支
        "$randomUserAgent" => pick(&[ // 匹配分支
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36", // 语句
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_0) AppleWebKit/605.1.15", // 语句
            "Mozilla/5.0 (X11; Linux x86_64) Gecko/20100101 Firefox/118.0", // 语句
        ]).to_string(), // 语句
        "$randomProtocol" => pick(&["http", "https"]).to_string(), // 匹配分支
        "$randomDomainName" => format!("{}.{}", pick(&["example", "acme", "test", "service"]), pick(&["com", "net", "org", "io"])), // 匹配分支
        "$randomDomainSuffix" => pick(&["com", "net", "org", "io"]).to_string(), // 匹配分支
        "$randomUrl" => format!("https://{}.{}/{}", pick(&["example", "acme", "test"]), pick(&["com", "io", "net"]), pick(&["api", "v1", "docs"])), // 匹配分支
        "$randomIPV6" => format!( // 匹配分支
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}", // 语句
            rng.gen_range(0..65535), rng.gen_range(0..65535), rng.gen_range(0..65535), rng.gen_range(0..65535), // 语句
            rng.gen_range(0..65535), rng.gen_range(0..65535), rng.gen_range(0..65535), rng.gen_range(0..65535) // 语句
        ), // 语句
        "$randomMACAddress" => format!( // 匹配分支
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", // 语句
            rng.gen_range(0..256), rng.gen_range(0..256), rng.gen_range(0..256), // 语句
            rng.gen_range(0..256), rng.gen_range(0..256), rng.gen_range(0..256) // 语句
        ), // 语句
        "$randomLocale" => pick(&["zh_CN", "en_US", "ja_JP", "de_DE", "fr_FR"]).to_string(), // 匹配分支
        "$randomSemver" => format!("{}.{}.{}", rng.gen_range(0..10), rng.gen_range(0..20), rng.gen_range(0..50)), // 匹配分支
        "$randomBankAccount" => format!("{:08}", rng.gen_range(0..100_000_000)), // 匹配分支
        "$randomCreditCardMask" => format!("**** **** **** {:04}", rng.gen_range(0..10000)), // 匹配分支
        "$randomDateFuture" => (chrono::Utc::now() + chrono::Duration::days(rng.gen_range(1..365))).to_rfc3339(), // 匹配分支
        "$randomDatePast" => (chrono::Utc::now() - chrono::Duration::days(rng.gen_range(1..365))).to_rfc3339(), // 匹配分支
        "$randomDateRecent" => (chrono::Utc::now() - chrono::Duration::hours(rng.gen_range(1..72))).to_rfc3339(), // 匹配分支
        "$randomLoremWord" => pick(&lorem_words), // 匹配分支
        "$randomLoremWords" => (0..5) // 匹配分支
            .map(|_| pick(&lorem_words)) // 语句
            .collect::<Vec<_>>() // 语句
            .join(" "), // 语句
        "$randomLoremSentence" => format!( // 匹配分支
            "{}.", // 语句
            (0..8).map(|_| pick(&lorem_words)).collect::<Vec<_>>().join(" ") // 语句
        ), // 语句
        "$randomLoremSentences" => (0..3) // 匹配分支
            .map(|_| { // 语句
                format!( // 语句
                    "{}.", // 语句
                    (0..8).map(|_| pick(&lorem_words)).collect::<Vec<_>>().join(" ") // 语句
                ) // 块结束
            }) // 语句
            .collect::<Vec<_>>() // 语句
            .join(" "), // 语句
        "$randomLoremParagraph" => (0..4) // 匹配分支
            .map(|_| { // 语句
                format!( // 语句
                    "{}.", // 语句
                    (0..10).map(|_| pick(&lorem_words)).collect::<Vec<_>>().join(" ") // 语句
                ) // 块结束
            }) // 语句
            .collect::<Vec<_>>() // 语句
            .join(" "), // 语句
        "$randomLoremParagraphs" => (0..3) // 匹配分支
            .map(|_| { // 语句
                (0..4) // 语句
                    .map(|_| { // 语句
                        format!( // 语句
                            "{}.", // 语句
                            (0..10).map(|_| pick(&lorem_words)).collect::<Vec<_>>().join(" ") // 语句
                        ) // 块结束
                    }) // 语句
                    .collect::<Vec<_>>() // 语句
                    .join(" ") // 语句
            }) // 语句
            .collect::<Vec<_>>() // 语句
            .join("\n\n"), // 语句
        "$randomLoremText" => (0..5) // 匹配分支
            .map(|_| { // 语句
                format!( // 语句
                    "{}.", // 语句
                    (0..9).map(|_| pick(&lorem_words)).collect::<Vec<_>>().join(" ") // 语句
                ) // 块结束
            }) // 语句
            .collect::<Vec<_>>() // 语句
            .join(" "), // 语句
        "$randomLoremSlug" => (0..4) // 匹配分支
            .map(|_| pick(&lorem_words)) // 语句
            .collect::<Vec<_>>() // 语句
            .join("-"), // 语句
        "$randomImageUrl" | "$randomAbstractImage" | "$randomAnimalsImage" | "$randomBusinessImage" // 语句
        | "$randomCatsImage" | "$randomCityImage" | "$randomFoodImage" | "$randomNightlifeImage" // 语句
        | "$randomFashionImage" | "$randomPeopleImage" | "$randomNatureImage" | "$randomSportsImage" // 语句
        | "$randomTransportImage" => format!( // 匹配分支
            "https://loremflickr.com/320/240/{}", // 语句
            pick(&["abstract", "animals", "business", "cats", "city", "food"]) // 调用 pick
        ), // 语句
        "$randomAvatar" => format!( // 匹配分支
            "https://i.pravatar.cc/150?u={}", // 语句
            uuid::Uuid::new_v4().simple() // 调用 new_v4
        ), // 语句
        "$randomUserName" => format!( // 匹配分支
            "{}.{}{}", // 语句
            pick(&first).to_lowercase(), // 调用 pick
            pick(&last).to_lowercase(), // 调用 pick
            rng.gen_range(1..99) // 语句
        ), // 语句
        "$randomBankAccountName" => pick(&first), // 匹配分支
        "$randomBankAccountIban" | "$randomIban" => format!( // 匹配分支
            "{}{}{}", // 语句
            pick(&["DE", "GB", "FR", "CN"]), // 调用 pick
            rng.gen_range(10..99), // 语句
            (0..16) // 语句
                .map(|_| rng.gen_range(0..10).to_string()) // 语句
                .collect::<Vec<_>>() // 语句
                .join("") // 语句
        ), // 语句
        "$randomBankAccountBic" | "$randomBic" => format!( // 匹配分支
            "{}{}XXXX", // 语句
            pick(&["DEUT", "BNPA", "CITI", "HSBC"]), // 调用 pick
            pick(&["DE", "GB", "FR", "CN"]) // 调用 pick
        ), // 语句
        "$randomBankAccountNumber" => format!("{:08}", rng.gen_range(0..100_000_000)), // 匹配分支
        "$randomBankCountry" => pick(&countries), // 匹配分支
        "$randomCreditCardNumber" => (0..16) // 匹配分支
            .map(|_| rng.gen_range(0..10).to_string()) // 语句
            .collect::<Vec<_>>() // 语句
            .join(""), // 语句
        "$randomAmount" => format!("{}.{:02}", rng.gen_range(0..10000), rng.gen_range(0..100)), // 匹配分支
        _ => return None, // 匹配分支
    }; // 块结束
    Some(v) // 调用 Some
} // 块结束

#[cfg(test)] // 语句
mod tests { // 模块 tests
    use super::*; // 引入依赖

    fn ctx() -> VarContext { // 函数 ctx
        let mut c = VarContext::new(); // 变量 c
        let mut globals = HashMap::new(); // 变量 globals
        globals.insert("host".to_string(), "https://api.test".to_string()); // 语句
        let mut env = HashMap::new(); // 变量 env
        env.insert("host".to_string(), "https://env.test".to_string()); // 语句
        env.insert("path".to_string(), "users".to_string()); // 语句
        // scopes are ordered high -> low; environment must beat globals
        c.scopes.push(env); // 语句
        c.scopes.push(globals); // 语句
        c // 语句
    } // 块结束

    #[test] // 语句
    fn resolves_and_prioritizes() { // 函数 resolves_and_prioritizes
        let c = ctx(); // 变量 c
        assert_eq!(c.resolve("{{host}}/{{path}}"), "https://env.test/users"); // 语句
    } // 块结束

    #[test] // 语句
    fn recursive_resolution() { // 函数 recursive_resolution
        let mut c = VarContext::new(); // 变量 c
        let mut env = HashMap::new(); // 变量 env
        env.insert("a".to_string(), "{{b}}".to_string()); // 语句
        env.insert("b".to_string(), "final".to_string()); // 语句
        c.push_low(env); // 语句
        assert_eq!(c.resolve("{{a}}"), "final"); // 语句
    } // 块结束

    #[test] // 语句
    fn dynamic_vars() { // 函数 dynamic_vars
        assert!(dynamic_value("$randomInt").unwrap().parse::<i32>().is_ok()); // 语句
        assert_eq!(dynamic_value("$randomBoolean").unwrap().len() >= 4, true); // 语句
        assert!(dynamic_value("$nonexistent").is_none()); // 语句
    } // 块结束

    #[test] // 语句
    fn leaves_unknown_intact() { // 函数 leaves_unknown_intact
        let c = ctx(); // 变量 c
        assert_eq!(c.resolve("{{missing}}"), "{{missing}}"); // 语句
    } // 块结束

    #[test] // 语句
    fn lorem_and_image_dynamic_vars() { // 函数 lorem_and_image_dynamic_vars
        assert!(dynamic_value("$randomLoremSentence").unwrap().ends_with('.')); // 语句
        assert!(dynamic_value("$randomImageUrl").unwrap().starts_with("https://")); // 语句
        assert_eq!(dynamic_value("$randomCreditCardNumber").unwrap().len(), 16); // 语句
        assert!(dynamic_value("$randomLoremSlug").unwrap().contains('-')); // 语句
        assert!(!dynamic_value("$randomAvatar").unwrap().is_empty()); // 语句
    } // 块结束
} // 块结束
