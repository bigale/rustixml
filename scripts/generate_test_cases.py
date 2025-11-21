#!/usr/bin/env python3
"""
Extract passing iXML test cases and generate JSON for demo pages.

This script reads test cases from ixml_tests/correct/ and creates
a JSON file with curated examples for the interactive demos.
"""

import json
import os
from pathlib import Path
from typing import Dict, List, Optional

# Test cases to skip (grammar parsing not yet supported or too advanced)
SKIP_TESTS = {
    'unicode-version-diagnostic',
    'version-decl',
    'version-decl.2',
    'ws-and-delim',
}

# Category mapping for organization
CATEGORIES = {
    'Basic Examples': [
        'aaa', 'test', 'empty-group', 'lf', 'tab',
    ],
    'Arithmetic & Math': [
        'arith', 'expr', 'expr1', 'expr2', 'expr3', 'expr4', 'expr5', 'expr6',
        'poly', 'hash',
    ],
    'Data Formats': [
        'json', 'json1', 'xml', 'xml1', 'vcard', 'diary', 'diary2', 'diary3',
    ],
    'Email & Addresses': [
        'email', 'address',
    ],
    'Text Processing': [
        'string', 'marked', 'para-test', 'nested-comment', 'range-comments',
        'element-content', 'attribute-value',
    ],
    'Character Classes': [
        'range', 'ranges', 'ranges1', 'hex', 'hex1', 'hex3',
        'unicode-classes', 'unicode-range', 'unicode-range1', 'unicode-range2',
    ],
    'Programming Languages': [
        'program', 'xpath',
    ],
}

def get_category(test_name: str) -> str:
    """Find which category a test belongs to."""
    for category, tests in CATEGORIES.items():
        if test_name in tests:
            return category
    return 'Other'

def read_file_safe(path: Path) -> Optional[str]:
    """Read file, return None if doesn't exist."""
    try:
        return path.read_text(encoding='utf-8').strip()
    except (FileNotFoundError, UnicodeDecodeError):
        return None

def get_description(test_name: str) -> str:
    """Generate a human-readable description for a test."""
    descriptions = {
        'arith': 'Parenthesized arithmetic expression with operators',
        'email': 'Email address validation with complex character classes',
        'address': 'Postal address parsing',
        'json': 'Simple JSON parser',
        'json1': 'JSON with nested objects',
        'xml': 'Basic XML parser',
        'xml1': 'XML with attributes',
        'expr': 'Expression with precedence',
        'diary': 'Diary entry with date parsing',
        'vcard': 'vCard contact format',
        'program': 'Simple programming language',
        'xpath': 'XPath expression parsing',
        'hex': 'Hexadecimal number parsing',
        'string': 'String literal parsing',
        'range': 'Character range matching',
        'unicode-classes': 'Unicode character class support',
    }
    return descriptions.get(test_name, f'{test_name.replace("-", " ").title()}')

def extract_test_cases(tests_dir: Path) -> Dict[str, List[Dict]]:
    """Extract all passing test cases from ixml_tests/correct/."""
    
    categories: Dict[str, List[Dict]] = {}
    
    # Find all .ixml files
    for ixml_file in sorted(tests_dir.glob('*.ixml')):
        test_name = ixml_file.stem
        
        # Skip unsupported tests
        if test_name in SKIP_TESTS:
            print(f"⏭️  Skipping {test_name} (not yet supported)")
            continue
        
        # Read grammar
        grammar = read_file_safe(ixml_file)
        if not grammar:
            print(f"⚠️  Skipping {test_name} (no grammar)")
            continue
        
        # Read input
        inp_file = ixml_file.with_suffix('.inp')
        input_text = read_file_safe(inp_file)
        if not input_text:
            print(f"⚠️  Skipping {test_name} (no input)")
            continue
        
        # Get category
        category = get_category(test_name)
        
        # Create test case
        test_case = {
            'name': test_name.replace('-', ' ').title(),
            'id': test_name,
            'grammar': grammar,
            'input': input_text,
            'description': get_description(test_name),
        }
        
        # Add to category
        if category not in categories:
            categories[category] = []
        categories[category].append(test_case)
        
        print(f"✅ Added {test_name} to '{category}'")
    
    return categories

def main():
    # Paths
    repo_root = Path(__file__).parent.parent
    tests_dir = repo_root / 'ixml_tests' / 'correct'
    output_file = repo_root / 'docs' / 'test-cases.json'
    
    print(f"📁 Reading tests from: {tests_dir}")
    print(f"📝 Output file: {output_file}")
    print()
    
    # Extract test cases
    categories = extract_test_cases(tests_dir)
    
    # Sort categories (Basic Examples first, Other last)
    category_order = ['Basic Examples', 'Arithmetic & Math', 'Data Formats', 
                      'Email & Addresses', 'Text Processing', 'Character Classes',
                      'Programming Languages', 'Other']
    sorted_categories = {
        cat: categories[cat] 
        for cat in category_order 
        if cat in categories
    }
    
    # Create output structure
    output = {
        'version': '0.2.0',
        'description': 'Passing test cases from the iXML test suite',
        'categories': sorted_categories,
        'stats': {
            'total_tests': sum(len(tests) for tests in categories.values()),
            'categories': len(categories),
        }
    }
    
    # Write JSON
    output_file.parent.mkdir(exist_ok=True)
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(output, f, indent=2, ensure_ascii=False)
    
    print()
    print(f"✨ Generated {output['stats']['total_tests']} test cases in {output['stats']['categories']} categories")
    print(f"📦 Saved to: {output_file}")
    
    # Print summary
    print("\n📊 Summary by category:")
    for category, tests in sorted_categories.items():
        print(f"  {category}: {len(tests)} tests")

if __name__ == '__main__':
    main()
