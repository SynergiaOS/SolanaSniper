# Prosty test do sprawdzenia czy pytest działa

def test_simple_math():
    """Prosty test matematyczny"""
    assert 2 + 2 == 4

def test_string_operations():
    """Test operacji na stringach"""
    text = "Solana price surges"
    assert "solana" in text.lower()
    assert "price" in text.lower()

def test_list_operations():
    """Test operacji na listach"""
    keywords = ["solana", "sol", "defi"]
    text = "Solana DeFi protocol launches"
    
    matches = sum(1 for keyword in keywords if keyword in text.lower())
    assert matches >= 2  # "solana" i "defi"

if __name__ == "__main__":
    import pytest
    pytest.main([__file__, "-v"])
