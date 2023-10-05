
def compare(a,b)
  # return a == b
  return false unless a.class == b.class
  case a
  when Float
    return (a-b).abs() < 1e-12
  when Array
    return false unless a.size == b.size
    a.zip(b) do |va,vb|
      return false unless compare(va, vb)
    end
    return true
  when Hash
    return false unless a.keys.sort == b.keys.sort
    if a.keys.sort == ["labels", "properties", "type"] && a["type"] == "node"
        return a["labels"].sort == b["labels"].sort && compare(a["properties"], b["properties"])
    else
      a.keys.each do |k|
        return false unless compare(a[k], b[k])
      end
      return true
    end
  else
    return a == b
  end
  raise "Unhandled comparison between #{a} and #{b}"
end

def compare_table_in_any_order(actual, expected)
  return false unless actual.length == expected.length
  # Resort the column so that they match
  unless actual[0] == expected[0]
    return false unless actual[0].sort == expected[0].sort
    sort_order = []
    actual[0].each do |e|
      sort_order.push expected[0].index(e)
    end
    expected = expected.map do |r|
      nr = []
      sort_order.each do |idx|
        nr.push r[idx]
      end
      nr
    end
  end
  # Find matches
  for i in 1...actual.length
    match = false
    for j in 1...actual.length
      if compare(actual[i], expected[j])
        match = true
        break
      end
    end
    return false unless match
  end
  return true
end
