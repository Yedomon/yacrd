/*
Copyright (c) 2018 Pierre Marijon <pierre.marijon@inria.fr>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/* crates use */
use clap;

/* standard use */

#[derive(Debug, PartialEq)]
pub enum Format {
    Paf,
    Mhap,
    Fasta,
    Fastq,
}

pub fn get_mapping_format(matches: &clap::ArgMatches, formats: &mut Vec<Format>) {
    if matches.is_present("format") {
        if matches.values_of("input").unwrap().len() > 1 {
            panic!("Format option are avaible only for one input");
        } else {
            formats.push(match matches.value_of("format").unwrap() {
                "paf" => Format::Paf,
                "mhap" => Format::Mhap,
                _ => panic!("You can't be her send your command line to pierre.marijon@inria.fr"),
            })
        }
    } else {
        for input_name in matches.values_of("input").unwrap() {
            formats.push(
                get_format_mapping(input_name).expect("Format of input can be determinate check file extension (paf and mhap are allowed)")
            );
        }
    }
}

pub fn get_format_mapping(filename: &str) -> Option<Format> {
    return match get_format(filename) {
        e @ Some(Format::Paf) |
        e @ Some(Format::Mhap) => e,
        _ => None,
    };
}

pub fn get_format(filename: &str) -> Option<Format> {
    return if filename == "-" {
        Some(Format::Paf)
    } else if filename.contains(".paf") {
        Some(Format::Paf)
    } else if filename.contains(".mhap") {
        Some(Format::Mhap)
    } else if filename.contains(".fasta") {
        Some(Format::Fasta)
    } else if filename.contains(".fastq") {
        Some(Format::Fastq)
    } else {
        None
    };
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn filename_mapping_only() {
        assert_eq!(get_format_mapping("t.paf").unwrap(), Format::Paf);
        assert_eq!(get_format_mapping("t.mhap").unwrap(), Format::Mhap);
        assert_eq!(get_format_mapping("t.fasta"), None);
        assert_eq!(get_format_mapping("t.fastq"), None);
    }

    #[test]
    fn filename_based() {
        assert_eq!(get_format("t.paf").unwrap(), Format::Paf);
        assert_eq!(get_format("t.mhap").unwrap(), Format::Mhap);
        assert_eq!(get_format("t.fasta").unwrap(), Format::Fasta);
        assert_eq!(get_format("t.fastq").unwrap(), Format::Fastq);
    }

    #[test]
    fn filename_based_compressed() {
        assert_eq!(get_format("t.paf.gz").unwrap(), Format::Paf);
        assert_eq!(get_format("t.mhap.xz").unwrap(), Format::Mhap);
        assert_eq!(get_format("t.fasta.something").unwrap(), Format::Fasta);
        assert_eq!(get_format("t.fastq.zip").unwrap(), Format::Fastq);
    }

}
