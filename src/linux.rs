use failure;
use std::fs;

#[derive(Copy, Clone, Debug)]
pub enum Distribution {
    Arch,
    CentOS,
    Fedora,
    Debian,
    Ubuntu,
}

#[derive(Debug, Fail)]
#[fail(display = "Unknown Linux Distribution")]
struct UnknownLinuxDistribution;

impl Distribution {
    pub fn detect() -> Result<Self, failure::Error> {
        let content = fs::read_to_string("/etc/os-release")?;

        if content.contains("Arch") {
            return Ok(Distribution::Arch);
        }

        if content.contains("CentOS") {
            return Ok(Distribution::CentOS);
        }

        if content.contains("Fedora") {
            return Ok(Distribution::Fedora);
        }

        if content.contains("Ubuntu") {
            return Ok(Distribution::Ubuntu);
        }

        if content.contains("Debian") {
            return Ok(Distribution::Debian);
        }

        Err(UnknownLinuxDistribution.into())
    }
}
