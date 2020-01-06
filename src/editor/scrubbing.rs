/*
Copyright (c) 2020 Pierre Marijon <pmarijon@mpi-inf.mpg.de>

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

/* crate use */
use anyhow::{anyhow, Context, Result};
use bio::io::{fasta, fastq};

/* local use */
use editor;
use error;
use stack;
use util;

pub fn scrubbing(
    input_path: &str,
    output_path: &str,
    badregions: &mut dyn stack::BadPart,
    not_covered: f64,
) -> Result<()> {
    let (input, compression) = util::read_file(input_path)?;
    let output = util::write_file(output_path, compression)?;

    match util::get_file_type(input_path) {
        Some(util::FileType::Fasta) => fasta(input, output, badregions, not_covered)
            .with_context(|| anyhow!("Filename: {}", input_path.to_string()))?,
        Some(util::FileType::Fastq) => fastq(input, output, badregions, not_covered)
            .with_context(|| anyhow!("Filename: {}", input_path.to_string()))?,
        Some(util::FileType::Paf) => bail!(error::Error::CantRunOperationOnFile {
            operation: "scrubbing".to_string(),
            filetype: util::FileType::Paf,
            filename: input_path.to_string()
        }),
        Some(util::FileType::M4) => bail!(error::Error::CantRunOperationOnFile {
            operation: "scrubbing".to_string(),
            filetype: util::FileType::M4,
            filename: input_path.to_string()
        }),
        Some(util::FileType::Yacrd) => bail!(error::Error::CantRunOperationOnFile {
            operation: "scrubbing".to_string(),
            filetype: util::FileType::Yacrd,
            filename: input_path.to_string()
        }),
        None | Some(util::FileType::YacrdOverlap) => {
            bail!(error::Error::UnableToDetectFileFormat {
                filename: input_path.to_string()
            })
        }
    };

    Ok(())
}

fn fasta<R, W>(
    input: R,
    output: W,
    badregions: &mut dyn stack::BadPart,
    not_covered: f64,
) -> Result<()>
where
    R: std::io::Read,
    W: std::io::Write,
{
    let reader = fasta::Reader::new(input);
    let mut writer = fasta::Writer::new(output);

    for result in reader.records() {
        let record = result.with_context(|| error::Error::ReadingErrorNoFilename {
            format: util::FileType::Fasta,
        })?;

        let (badregion, length) = badregions.get_bad_part(&record.id().to_string())?;

        let rtype = editor::type_of_read(*length, badregion, not_covered);

        if rtype == editor::ReadType::NotCovered {
            continue;
        } else if badregion.len() == 0 {
            writer
                .write_record(&record)
                .with_context(|| error::Error::WritingErrorNoFilename {
                    format: util::FileType::Fasta,
                })?;
        } else {
            let mut poss = vec![0];
            for interval in badregion {
                poss.push(interval.0);
                poss.push(interval.1);
            }

	    if poss.last() != Some(&(*length as u32)) {
		poss.push(*length as u32);
	    };
	    
	    let iter = if poss[0] == 0 && poss[1] == 0 { &poss[2..] } else { &poss[..] };
	    
	    for pos in iter.chunks_exact(2) {
		writer
                    .write(
                        &format!("{}_{}_{}", record.id(), pos[0], pos[1]),
                        record.desc(),
                        &record.seq()[(pos[0] as usize)..(pos[1] as usize)],
                    )
                    .with_context(|| error::Error::WritingErrorNoFilename {
                        format: util::FileType::Fasta,
                    })?;
            }
        }
    }

    Ok(())
}

fn fastq<R, W>(
    input: R,
    output: W,
    badregions: &mut dyn stack::BadPart,
    not_covered: f64,
) -> Result<()>
where
    R: std::io::Read,
    W: std::io::Write,
{
    let reader = fastq::Reader::new(input);
    let mut writer = fastq::Writer::new(output);

    for result in reader.records() {
        let record = result.with_context(|| error::Error::ReadingErrorNoFilename {
            format: util::FileType::Fastq,
        })?;

        let (badregion, length) = badregions.get_bad_part(&record.id().to_string())?;

        let rtype = editor::type_of_read(*length, badregion, not_covered);

        if rtype == editor::ReadType::NotCovered {
            continue;
        } else if badregion.len() == 0 {
            writer
                .write_record(&record)
                .with_context(|| error::Error::WritingErrorNoFilename {
                    format: util::FileType::Fastq,
                })?;
        } else {
            let mut poss = vec![0];
            for interval in badregion {
                poss.push(interval.0);
                poss.push(interval.1);
            }
            
	    if poss.last() != Some(&(*length as u32)) {
		poss.push(*length as u32);
	    };
	    
	    let iter = if poss[0] == 0 && poss[1] == 0 { &poss[2..] } else { &poss[..] };
	    
            for pos in iter.chunks_exact(2) {
                writer
                    .write(
                        &format!("{}_{}_{}", record.id(), pos[0], pos[1]),
                        record.desc(),
                        &record.seq()[(pos[0] as usize)..(pos[1] as usize)],
                        &record.qual()[(pos[0] as usize)..(pos[1] as usize)],
                    )
                    .with_context(|| error::Error::WritingErrorNoFilename {
                        format: util::FileType::Fastq,
                    })?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use reads2ovl;
    use reads2ovl::Reads2Ovl;

    const FASTA_FILE: &'static [u8] = b">1
ACTGGGGGGACTGGGGGGACTG
>2
ACTG
>3
ACTG
";

    const FASTA_FILE_SCRUBBED: &'static [u8] = b">1_0_4
ACTG
>1_9_13
ACTG
>1_18_22
ACTG
>2
ACTG
>3
ACTG
";

    #[test]
    fn fasta_keep_begin_end() -> () {
	let mut ovlst = reads2ovl::FullMemory::new();

	ovlst.add_length("1".to_string(), 22);
	ovlst.add_overlap("1".to_string(), (0, 4)).unwrap();
	ovlst.add_overlap("1".to_string(), (9, 13)).unwrap();
	ovlst.add_overlap("1".to_string(), (18, 22)).unwrap();
	
	let mut stack = stack::FromOverlap::new(Box::new(ovlst), 0);

	let mut output: Vec<u8> = Vec::new();
	fasta(FASTA_FILE, &mut output, &mut stack, 0.8).unwrap();

	assert_eq!(FASTA_FILE_SCRUBBED, &output[..]);
    }

        const FASTA_FILE_SCRUBBED2: &'static [u8] = b">1_4_18
GGGGGACTGGGGGG
>2
ACTG
>3
ACTG
";

    #[test]
    fn fasta_keep_middle() -> () {
	let mut ovlst = reads2ovl::FullMemory::new();

	ovlst.add_length("1".to_string(), 22);
	ovlst.add_overlap("1".to_string(), (4, 18)).unwrap();
	
	let mut stack = stack::FromOverlap::new(Box::new(ovlst), 0);

	let mut output: Vec<u8> = Vec::new();
	fasta(FASTA_FILE, &mut output, &mut stack, 0.8).unwrap();

	assert_eq!(FASTA_FILE_SCRUBBED2, &output[..]);
    }

    
    const FASTQ_FILE: &'static [u8] = b"@1
ACTGGGGGGACTGGGGGGACTG
+
??????????????????????
@2
ACTG
+
????
@3
ACTG
+
????
";

    const FASTQ_FILE_SCRUBBED: &'static [u8] = b"@1_0_4
ACTG
+
????
@1_9_13
ACTG
+
????
@1_18_22
ACTG
+
????
@2
ACTG
+
????
@3
ACTG
+
????
";

    #[test]
    fn fastq_keep_begin_end() {
	let mut ovlst = reads2ovl::FullMemory::new();
	
	ovlst.add_length("1".to_string(), 22);
	ovlst.add_overlap("1".to_string(), (0, 4)).unwrap();
	ovlst.add_overlap("1".to_string(), (9, 13)).unwrap();
	ovlst.add_overlap("1".to_string(), (18, 22)).unwrap();
	
	let mut stack = stack::FromOverlap::new(Box::new(ovlst), 0);

	let mut output: Vec<u8> = Vec::new();
	fastq(FASTQ_FILE, &mut output, &mut stack, 0.8).unwrap();

	assert_eq!(FASTQ_FILE_SCRUBBED, &output[..]);
    }

    const FASTQ_FILE_SCRUBBED2: &'static [u8] = b"@1_4_18
GGGGGACTGGGGGG
+
??????????????
@2
ACTG
+
????
@3
ACTG
+
????
";
    
    #[test]
    fn fastq_keep_middle() {
	let mut ovlst = reads2ovl::FullMemory::new();
	
	ovlst.add_length("1".to_string(), 22);
	ovlst.add_overlap("1".to_string(), (4, 18)).unwrap();
	
	let mut stack = stack::FromOverlap::new(Box::new(ovlst), 0);

	let mut output: Vec<u8> = Vec::new();
	fastq(FASTQ_FILE, &mut output, &mut stack, 0.8).unwrap();

	assert_eq!(FASTQ_FILE_SCRUBBED2, &output[..]);
    }
}
