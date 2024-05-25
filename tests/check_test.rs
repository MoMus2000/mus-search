#[cfg(test)]
mod tests {
    use mus_search::add;

    #[test]
    fn test_add() {
        assert_eq!(add(4, 5), 9);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-2, -3), -5);
    }
}