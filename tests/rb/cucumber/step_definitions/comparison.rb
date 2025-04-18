
def compare(a,b)
  # return a == b
  if a.class == b.class
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
      if (a.keys.sort == ["labels", "properties", "type"] || a.keys.sort == ["key", "labels", "properties", "type"]) && (a["type"] == "node" || a["type"] == "edge")  && (b["type"] == "node" || b["type"] == "edge")
        return a["labels"].sort == b["labels"].sort && compare(a["properties"], b["properties"])
      elsif (a.keys.sort == ["destination", "labels", "properties", "source", "type"] || a.keys.sort == ["destination", "key", "labels", "properties", "source", "type"]) && (a["type"] == "path") && (b["type"] == "path")
        return a["labels"].sort == b["labels"].sort && compare(a["properties"], b["properties"]) && compare(a["source"], b["source"]) && compare(a["destination"], b["destination"])
      else
        return false unless a.keys.sort == b.keys.sort
        a.keys.each do |k|
          return false unless compare(a[k], b[k])
        end
        return true
      end
    else
      return a == b
    end
    raise "Unhandled comparison between #{a} and #{b}"
  else
    if a.class == Integer 
      case b
      when Float
        return (a-b).abs() < 1e-12
      when TrueClass
        return a == 1
      when FalseClass
        return a == 0
      else
        return false
      end
    elsif b.class == Integer
      case a
      when Float
        return (a-b).abs() < 1e-12
      when TrueClass
        return b == 1
      when FalseClass
        return b == 0
      else
        return false
      end
    else
      return false
    end
  end
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
    if !match
      puts("Mo match for #{actual[i]}")
      return false
    end
  end
  return true
end

def compare_table_in_order(actual, expected)
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
    unless compare(actual[i], expected[i])
      puts("actual '#{actual[i]}' != '#{expected[i]}'")
      return false
    end
  end
  return true
end
