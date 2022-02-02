use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

pub struct Wordlist {
    pub path: PathBuf,
    pub base_count: usize,
    pub total_count: usize,
    reader: BufReader<File>,
    pub prepend: Vec<String>,
    pub append: Vec<String>,
    pub swap: Vec<String>,
    pub extensions: Vec<String>,
    word_perms: VecDeque<String>,
}

fn count_lines<R: io::Read>(handle: R) -> usize {
    let mut reader = BufReader::new(handle);
    let mut count = 0;
    let mut line: Vec<u8> = Vec::new();
    while match reader.read_until(b'\n', &mut line) {
        Ok(n) if n > 0 => true,
        Err(e) => {
            eprintln!("[!] Failed to read from wordlist: {}", e);
            std::process::exit(-1);
        }
        _ => false,
    } {
        if *line.last().unwrap() == b'\n' {
            count += 1;
        };
    }
    count
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

impl Wordlist {
    pub fn new(
        path: &PathBuf,
        prepend: Option<String>,
        append: Option<String>,
        swap: Option<String>,
        extensions: Option<String>,
    ) -> Self {
        let pre_strs = match prepend {
            Some(s) => s.split(",").map(|s| s.to_string()).collect::<Vec<String>>(),
            None => vec![],
        };
        let pre_len = pre_strs.len();
        let app_strs = match append {
            Some(s) => s.split(",").map(|s| s.to_string()).collect::<Vec<String>>(),
            None => vec![],
        };
        let app_len = app_strs.len();
        let ext_strs = match extensions {
            Some(s) => s.split(",").map(|s| s.to_string()).collect::<Vec<String>>(),
            None => vec![],
        };
        let ext_len = ext_strs.len();
        let swap_strs = match swap {
            Some(s) => s.split(",").map(|s| s.to_string()).collect::<Vec<String>>(),
            None => vec![],
        };
        let word_count = count_lines(std::fs::File::open(&path).unwrap());
        Self {
            path: path.clone(),
            base_count: word_count,
            reader: BufReader::new(File::open(path).unwrap()),
            prepend: pre_strs,
            append: app_strs,
            swap: swap_strs,
            extensions: ext_strs,
            total_count: word_count
                + (word_count * pre_len)
                + (word_count * app_len)
                + (word_count * ext_len),
            word_perms: VecDeque::new(),
        }
    }
}

impl Iterator for Wordlist {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.word_perms.is_empty() {
            let mut base_word = String::new();
            match self.reader.read_line(&mut base_word) {
                Ok(n) => {
                    if n != 0 {
                        trim_newline(&mut base_word);

                        // handle swap and base word
                        // words with swap are ignored if no swap keys provided
                        if base_word.contains("{SWAP}") {
                            for s in &self.swap {
                                self.word_perms
                                    .push_back(base_word.clone().replace("{SWAP}", &s))
                            }
                        } else {
                            self.word_perms.push_back(base_word.clone());
                        }

                        // handle prepends
                        for i in 0..self.word_perms.len() {
                            for p in &self.prepend {
                                self.word_perms
                                    .push_back(format!("{}{}", p, self.word_perms[i]));
                            }
                        }

                        // handle appends
                        for i in 0..self.word_perms.len() {
                            for a in &self.append {
                                self.word_perms
                                    .push_back(format!("{}{}", self.word_perms[i], a));
                            }
                        }

                        // handle extensions
                        for i in 0..self.word_perms.len() {
                            for e in &self.extensions {
                                self.word_perms
                                    .push_back(format!("{}{}", self.word_perms[i], e));
                            }
                        }
                    } else {
                        return None;
                    }
                }
                Err(_) => return None,
            }
        }
        self.word_perms.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use ctor;
    use std::io::Write;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    const WL_PATH: &str = "/tmp/rustbuster_test_wordlist.txt";

    #[ctor::ctor]
    fn gen_test_wordlist() {
        let lines_vec = vec!["test", "line2", "{SWAP}stest"];
        let tfile = std::fs::File::create(WL_PATH).unwrap();
        let mut writer = std::io::BufWriter::new(tfile);

        for l in lines_vec {
            writer.write_all(l.as_bytes()).unwrap();
            writer.write(b"\n").unwrap();
        }
        writer.flush().unwrap();
    }

    #[ctor::dtor]
    fn cleanup_wordlist() {
        std::fs::remove_file(WL_PATH).unwrap()
    }

    fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
        matching == a.len() && matching == b.len()
    }

    #[test]
    fn test_count_lines() {
        let handle = std::fs::File::open(WL_PATH).unwrap();
        let count = count_lines(handle);

        println!("count_lines: {}", count);

        assert!(count == 3);
    }

    #[test]
    fn test_trim_newline() {
        let mut line1 = String::from("line1\n");
        let mut line2 = String::from("line2\r\n");

        trim_newline(&mut line1);
        trim_newline(&mut line2);

        assert!(line1 == "line1".to_string());
        assert!(line2 == "line2".to_string());
    }

    #[test]
    fn test_prepend() {
        let pb = std::path::PathBuf::from(WL_PATH);
        let prepend = String::from("test1,test2,test3");
        let wl = Wordlist::new(&pb, Some(prepend), None, None, None);

        let words = wl.collect::<Vec<String>>();
        let answer = vec![
            "test".to_string(),
            "test1test".to_string(),
            "test2test".to_string(),
            "test3test".to_string(),
            "line2".to_string(),
            "test1line2".to_string(),
            "test2line2".to_string(),
            "test3line2".to_string(),
        ];

        println!("test_prepend: {:?}", words);
        assert!(do_vecs_match(&words, &answer));
    }

    #[test]
    fn test_append() {
        let pb = std::path::PathBuf::from(WL_PATH);
        let append = String::from("test1,test2,test3");
        let wl = Wordlist::new(&pb, None, Some(append), None, None);

        let words = wl.collect::<Vec<String>>();
        let answer = vec![
            "test".to_string(),
            "testtest1".to_string(),
            "testtest2".to_string(),
            "testtest3".to_string(),
            "line2".to_string(),
            "line2test1".to_string(),
            "line2test2".to_string(),
            "line2test3".to_string(),
        ];

        println!("test_append: {:?}", words);
        assert!(do_vecs_match(&words, &answer));
    }

    #[test]
    fn test_swap() {
        let pb = std::path::PathBuf::from(WL_PATH);
        let swap = String::from("dev,prod");
        let wl = Wordlist::new(&pb, None, None, Some(swap), None);

        let words = wl.collect::<Vec<String>>();
        let answer = vec![
            "test".to_string(),
            "line2".to_string(),
            "devstest".to_string(),
            "prodstest".to_string(),
        ];

        println!("test_swap: {:?}", words);
        assert!(do_vecs_match(&words, &answer));
    }

    #[test]
    fn test_extensions() {
        let pb = std::path::PathBuf::from(WL_PATH);
        let extensions = String::from(".txt,.bak,.file");
        let wl = Wordlist::new(&pb, None, None, None, Some(extensions));

        let words = wl.collect::<Vec<String>>();
        let answer = vec![
            "test".to_string(),
            "test.txt".to_string(),
            "test.bak".to_string(),
            "test.file".to_string(),
            "line2".to_string(),
            "line2.txt".to_string(),
            "line2.bak".to_string(),
            "line2.file".to_string(),
        ];

        println!("test_extensions: {:?}", words);
        assert!(do_vecs_match(&words, &answer));
    }

    #[test]
    fn test_all() {
        let pb = std::path::PathBuf::from(WL_PATH);
        let prepend = String::from("test1,test2,test3");
        let append = String::from("test1,test2,test3");
        let swap = String::from("dev,prod");
        let extensions = String::from(".txt,.bak,.file");

        let wl = Wordlist::new(
            &pb,
            Some(prepend),
            Some(append),
            Some(swap),
            Some(extensions),
        );

        let words = wl.collect::<Vec<String>>();
        let answer = vec![
            "test".to_string(),
            "test1test".to_string(),
            "test2test".to_string(),
            "test3test".to_string(),
            "testtest1".to_string(),
            "testtest2".to_string(),
            "testtest3".to_string(),
            "test1testtest1".to_string(),
            "test1testtest2".to_string(),
            "test1testtest3".to_string(),
            "test2testtest1".to_string(),
            "test2testtest2".to_string(),
            "test2testtest3".to_string(),
            "test3testtest1".to_string(),
            "test3testtest2".to_string(),
            "test3testtest3".to_string(),
            "test.txt".to_string(),
            "test.bak".to_string(),
            "test.file".to_string(),
            "test1test.txt".to_string(),
            "test1test.bak".to_string(),
            "test1test.file".to_string(),
            "test2test.txt".to_string(),
            "test2test.bak".to_string(),
            "test2test.file".to_string(),
            "test3test.txt".to_string(),
            "test3test.bak".to_string(),
            "test3test.file".to_string(),
            "testtest1.txt".to_string(),
            "testtest1.bak".to_string(),
            "testtest1.file".to_string(),
            "testtest2.txt".to_string(),
            "testtest2.bak".to_string(),
            "testtest2.file".to_string(),
            "testtest3.txt".to_string(),
            "testtest3.bak".to_string(),
            "testtest3.file".to_string(),
            "test1testtest1.txt".to_string(),
            "test1testtest1.bak".to_string(),
            "test1testtest1.file".to_string(),
            "test1testtest2.txt".to_string(),
            "test1testtest2.bak".to_string(),
            "test1testtest2.file".to_string(),
            "test1testtest3.txt".to_string(),
            "test1testtest3.bak".to_string(),
            "test1testtest3.file".to_string(),
            "test2testtest1.txt".to_string(),
            "test2testtest1.bak".to_string(),
            "test2testtest1.file".to_string(),
            "test2testtest2.txt".to_string(),
            "test2testtest2.bak".to_string(),
            "test2testtest2.file".to_string(),
            "test2testtest3.txt".to_string(),
            "test2testtest3.bak".to_string(),
            "test2testtest3.file".to_string(),
            "test3testtest1.txt".to_string(),
            "test3testtest1.bak".to_string(),
            "test3testtest1.file".to_string(),
            "test3testtest2.txt".to_string(),
            "test3testtest2.bak".to_string(),
            "test3testtest2.file".to_string(),
            "test3testtest3.txt".to_string(),
            "test3testtest3.bak".to_string(),
            "test3testtest3.file".to_string(),
            "line2".to_string(),
            "test1line2".to_string(),
            "test2line2".to_string(),
            "test3line2".to_string(),
            "line2test1".to_string(),
            "line2test2".to_string(),
            "line2test3".to_string(),
            "test1line2test1".to_string(),
            "test1line2test2".to_string(),
            "test1line2test3".to_string(),
            "test2line2test1".to_string(),
            "test2line2test2".to_string(),
            "test2line2test3".to_string(),
            "test3line2test1".to_string(),
            "test3line2test2".to_string(),
            "test3line2test3".to_string(),
            "line2.txt".to_string(),
            "line2.bak".to_string(),
            "line2.file".to_string(),
            "test1line2.txt".to_string(),
            "test1line2.bak".to_string(),
            "test1line2.file".to_string(),
            "test2line2.txt".to_string(),
            "test2line2.bak".to_string(),
            "test2line2.file".to_string(),
            "test3line2.txt".to_string(),
            "test3line2.bak".to_string(),
            "test3line2.file".to_string(),
            "line2test1.txt".to_string(),
            "line2test1.bak".to_string(),
            "line2test1.file".to_string(),
            "line2test2.txt".to_string(),
            "line2test2.bak".to_string(),
            "line2test2.file".to_string(),
            "line2test3.txt".to_string(),
            "line2test3.bak".to_string(),
            "line2test3.file".to_string(),
            "test1line2test1.txt".to_string(),
            "test1line2test1.bak".to_string(),
            "test1line2test1.file".to_string(),
            "test1line2test2.txt".to_string(),
            "test1line2test2.bak".to_string(),
            "test1line2test2.file".to_string(),
            "test1line2test3.txt".to_string(),
            "test1line2test3.bak".to_string(),
            "test1line2test3.file".to_string(),
            "test2line2test1.txt".to_string(),
            "test2line2test1.bak".to_string(),
            "test2line2test1.file".to_string(),
            "test2line2test2.txt".to_string(),
            "test2line2test2.bak".to_string(),
            "test2line2test2.file".to_string(),
            "test2line2test3.txt".to_string(),
            "test2line2test3.bak".to_string(),
            "test2line2test3.file".to_string(),
            "test3line2test1.txt".to_string(),
            "test3line2test1.bak".to_string(),
            "test3line2test1.file".to_string(),
            "test3line2test2.txt".to_string(),
            "test3line2test2.bak".to_string(),
            "test3line2test2.file".to_string(),
            "test3line2test3.txt".to_string(),
            "test3line2test3.bak".to_string(),
            "test3line2test3.file".to_string(),
            "devstest".to_string(),
            "prodstest".to_string(),
            "test1devstest".to_string(),
            "test2devstest".to_string(),
            "test3devstest".to_string(),
            "test1prodstest".to_string(),
            "test2prodstest".to_string(),
            "test3prodstest".to_string(),
            "devstesttest1".to_string(),
            "devstesttest2".to_string(),
            "devstesttest3".to_string(),
            "prodstesttest1".to_string(),
            "prodstesttest2".to_string(),
            "prodstesttest3".to_string(),
            "test1devstesttest1".to_string(),
            "test1devstesttest2".to_string(),
            "test1devstesttest3".to_string(),
            "test2devstesttest1".to_string(),
            "test2devstesttest2".to_string(),
            "test2devstesttest3".to_string(),
            "test3devstesttest1".to_string(),
            "test3devstesttest2".to_string(),
            "test3devstesttest3".to_string(),
            "test1prodstesttest1".to_string(),
            "test1prodstesttest2".to_string(),
            "test1prodstesttest3".to_string(),
            "test2prodstesttest1".to_string(),
            "test2prodstesttest2".to_string(),
            "test2prodstesttest3".to_string(),
            "test3prodstesttest1".to_string(),
            "test3prodstesttest2".to_string(),
            "test3prodstesttest3".to_string(),
            "devstest.txt".to_string(),
            "devstest.bak".to_string(),
            "devstest.file".to_string(),
            "prodstest.txt".to_string(),
            "prodstest.bak".to_string(),
            "prodstest.file".to_string(),
            "test1devstest.txt".to_string(),
            "test1devstest.bak".to_string(),
            "test1devstest.file".to_string(),
            "test2devstest.txt".to_string(),
            "test2devstest.bak".to_string(),
            "test2devstest.file".to_string(),
            "test3devstest.txt".to_string(),
            "test3devstest.bak".to_string(),
            "test3devstest.file".to_string(),
            "test1prodstest.txt".to_string(),
            "test1prodstest.bak".to_string(),
            "test1prodstest.file".to_string(),
            "test2prodstest.txt".to_string(),
            "test2prodstest.bak".to_string(),
            "test2prodstest.file".to_string(),
            "test3prodstest.txt".to_string(),
            "test3prodstest.bak".to_string(),
            "test3prodstest.file".to_string(),
            "devstesttest1.txt".to_string(),
            "devstesttest1.bak".to_string(),
            "devstesttest1.file".to_string(),
            "devstesttest2.txt".to_string(),
            "devstesttest2.bak".to_string(),
            "devstesttest2.file".to_string(),
            "devstesttest3.txt".to_string(),
            "devstesttest3.bak".to_string(),
            "devstesttest3.file".to_string(),
            "prodstesttest1.txt".to_string(),
            "prodstesttest1.bak".to_string(),
            "prodstesttest1.file".to_string(),
            "prodstesttest2.txt".to_string(),
            "prodstesttest2.bak".to_string(),
            "prodstesttest2.file".to_string(),
            "prodstesttest3.txt".to_string(),
            "prodstesttest3.bak".to_string(),
            "prodstesttest3.file".to_string(),
            "test1devstesttest1.txt".to_string(),
            "test1devstesttest1.bak".to_string(),
            "test1devstesttest1.file".to_string(),
            "test1devstesttest2.txt".to_string(),
            "test1devstesttest2.bak".to_string(),
            "test1devstesttest2.file".to_string(),
            "test1devstesttest3.txt".to_string(),
            "test1devstesttest3.bak".to_string(),
            "test1devstesttest3.file".to_string(),
            "test2devstesttest1.txt".to_string(),
            "test2devstesttest1.bak".to_string(),
            "test2devstesttest1.file".to_string(),
            "test2devstesttest2.txt".to_string(),
            "test2devstesttest2.bak".to_string(),
            "test2devstesttest2.file".to_string(),
            "test2devstesttest3.txt".to_string(),
            "test2devstesttest3.bak".to_string(),
            "test2devstesttest3.file".to_string(),
            "test3devstesttest1.txt".to_string(),
            "test3devstesttest1.bak".to_string(),
            "test3devstesttest1.file".to_string(),
            "test3devstesttest2.txt".to_string(),
            "test3devstesttest2.bak".to_string(),
            "test3devstesttest2.file".to_string(),
            "test3devstesttest3.txt".to_string(),
            "test3devstesttest3.bak".to_string(),
            "test3devstesttest3.file".to_string(),
            "test1prodstesttest1.txt".to_string(),
            "test1prodstesttest1.bak".to_string(),
            "test1prodstesttest1.file".to_string(),
            "test1prodstesttest2.txt".to_string(),
            "test1prodstesttest2.bak".to_string(),
            "test1prodstesttest2.file".to_string(),
            "test1prodstesttest3.txt".to_string(),
            "test1prodstesttest3.bak".to_string(),
            "test1prodstesttest3.file".to_string(),
            "test2prodstesttest1.txt".to_string(),
            "test2prodstesttest1.bak".to_string(),
            "test2prodstesttest1.file".to_string(),
            "test2prodstesttest2.txt".to_string(),
            "test2prodstesttest2.bak".to_string(),
            "test2prodstesttest2.file".to_string(),
            "test2prodstesttest3.txt".to_string(),
            "test2prodstesttest3.bak".to_string(),
            "test2prodstesttest3.file".to_string(),
            "test3prodstesttest1.txt".to_string(),
            "test3prodstesttest1.bak".to_string(),
            "test3prodstesttest1.file".to_string(),
            "test3prodstesttest2.txt".to_string(),
            "test3prodstesttest2.bak".to_string(),
            "test3prodstesttest2.file".to_string(),
            "test3prodstesttest3.txt".to_string(),
            "test3prodstesttest3.bak".to_string(),
            "test3prodstesttest3.file".to_string(),
        ];

        println!("test_all: {:?}", words);
        assert!(do_vecs_match(&words, &answer));
    }
}
