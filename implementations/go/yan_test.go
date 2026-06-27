package yan

import (
	"testing"
)

func TestPrimitiveString(t *testing.T) {
	parser := NewYANParser()
	result, err := parser.Parse("name: Budi")
	if err != nil {
		t.Fatal(err)
	}
	if result["name"] != "Budi" {
		t.Errorf("Expected 'Budi', got %v", result["name"])
	}
}

func TestPrimitiveNumber(t *testing.T) {
	parser := NewYANParser()
	result, err := parser.Parse("age: 25")
	if err != nil {
		t.Fatal(err)
	}
	if result["age"] != int64(25) {
		t.Errorf("Expected 25, got %v", result["age"])
	}
}

func TestBoolean(t *testing.T) {
	parser := NewYANParser()
	result, err := parser.Parse("active: yes")
	if err != nil {
		t.Fatal(err)
	}
	if result["active"] != true {
		t.Errorf("Expected true, got %v", result["active"])
	}
}

func TestNull(t *testing.T) {
	parser := NewYANParser()
	result, err := parser.Parse("data: null")
	if err != nil {
		t.Fatal(err)
	}
	if result["data"] != nil {
		t.Errorf("Expected nil, got %v", result["data"])
	}
}

func TestArray(t *testing.T) {
	parser := NewYANParser()
	result, err := parser.Parse("tags: a; b; c")
	if err != nil {
		t.Fatal(err)
	}
	arr, ok := result["tags"].([]YANValue)
	if !ok || len(arr) != 3 {
		t.Errorf("Expected array of 3, got %v", result["tags"])
	}
}

func TestInlineObject(t *testing.T) {
	parser := NewYANParser()
	result, err := parser.Parse("cfg: {host: localhost; port: 80}")
	if err != nil {
		t.Fatal(err)
	}
	obj, ok := result["cfg"].(map[string]YANValue)
	if !ok {
		t.Fatal("Expected object")
	}
	if obj["host"] != "localhost" {
		t.Errorf("Expected 'localhost', got %v", obj["host"])
	}
	if obj["port"] != int64(80) {
		t.Errorf("Expected 80, got %v", obj["port"])
	}
}

func TestBlockObject(t *testing.T) {
	parser := NewYANParser()
	result, err := parser.Parse("person:\n  name: Budi\n  age: 25")
	if err != nil {
		t.Fatal(err)
	}
	obj, ok := result["person"].(map[string]YANValue)
	if !ok {
		t.Fatal("Expected object")
	}
	if obj["name"] != "Budi" {
		t.Errorf("Expected 'Budi', got %v", obj["name"])
	}
}

func TestComment(t *testing.T) {
	parser := NewYANParser()
	result, err := parser.Parse("# comment\nname: Budi")
	if err != nil {
		t.Fatal(err)
	}
	if result["name"] != "Budi" {
		t.Errorf("Expected 'Budi', got %v", result["name"])
	}
}

func TestTypeHint(t *testing.T) {
	parser := NewYANParser()
	result, err := parser.Parse("n: @int 42")
	if err != nil {
		t.Fatal(err)
	}
	if result["n"] != int64(42) {
		t.Errorf("Expected 42, got %v", result["n"])
	}
}
