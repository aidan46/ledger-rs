mod intergration {
    use anyhow::Result;
    use assert_cmd::Command;
    use std::fs;

    #[test]
    fn simple() -> Result<()> {
        let mut cmd = Command::cargo_bin("csv_ledger")?;
        cmd.arg("./tests/data/simple.csv");
        cmd.arg("--sort");
        let cmd = cmd.unwrap();
        let output = String::from_utf8(cmd.stdout)?;
        let expected = fs::read_to_string("./tests/data/simple.out")?;
        let output = output
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();
        let expected = expected
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();

        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn locked() -> Result<()> {
        let mut cmd = Command::cargo_bin("csv_ledger")?;
        cmd.arg("./tests/data/locked.csv");
        cmd.arg("--sort");
        let cmd = cmd.unwrap();
        let output = String::from_utf8(cmd.stdout)?;
        let expected = fs::read_to_string("./tests/data/locked.out")?;
        let output = output
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();
        let expected = expected
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();

        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn duplicate_tx() -> Result<()> {
        let mut cmd = Command::cargo_bin("csv_ledger")?;
        cmd.arg("./tests/data/duplicate_tx.csv");
        cmd.arg("--sort");
        let cmd = cmd.unwrap();
        let output = String::from_utf8(cmd.stdout)?;
        let expected = fs::read_to_string("./tests/data/duplicate_tx.out")?;
        let output = output
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();
        let expected = expected
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();

        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn insufficient_funds() -> Result<()> {
        let mut cmd = Command::cargo_bin("csv_ledger")?;
        cmd.arg("./tests/data/insufficient_funds.csv");
        cmd.arg("--sort");
        let cmd = cmd.unwrap();
        let output = String::from_utf8(cmd.stdout)?;
        let expected = fs::read_to_string("./tests/data/insufficient_funds.out")?;
        let output = output
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();
        let expected = expected
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();

        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn resolve_dispute() -> Result<()> {
        let mut cmd = Command::cargo_bin("csv_ledger")?;
        cmd.arg("./tests/data/resolve_dispute.csv");
        cmd.arg("--sort");
        let cmd = cmd.unwrap();
        let output = String::from_utf8(cmd.stdout)?;
        let expected = fs::read_to_string("./tests/data/resolve_dispute.out")?;
        let output = output
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();
        let expected = expected
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();

        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn rescale() -> Result<()> {
        let mut cmd = Command::cargo_bin("csv_ledger")?;
        cmd.arg("./tests/data/rescale.csv");
        cmd.arg("--sort");
        let cmd = cmd.unwrap();
        let output = String::from_utf8(cmd.stdout)?;
        let expected = fs::read_to_string("./tests/data/rescale.out")?;
        let output = output
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();
        let expected = expected
            .chars()
            .filter(|c| !c.is_whitespace() || c == &'\n')
            .collect::<String>();

        assert_eq!(output, expected);
        Ok(())
    }
}
