require 'rspec'

def compare(a,b, ignore_list_order)
  # return a == b
  if a.class == b.class
    case a
    when Float
      return (a-b).abs() < 1e-12
    when Array
      return false unless a.size == b.size
      if ignore_list_order
        a.sort!.zip(b.sort!) do |va,vb|
          return false unless compare(va, vb, ignore_list_order)
        end
      else
        a.zip(b) do |va,vb|
          return false unless compare(va, vb, ignore_list_order)
        end
      end
      return true
    when Hash
      if (a.keys.sort == ["labels", "properties", "type"] || a.keys.sort == ["key", "labels", "properties", "type"]) && (a["type"] == "node" || a["type"] == "edge")  && (b["type"] == "node" || b["type"] == "edge")
        return a["labels"].sort == b["labels"].sort && compare(a["properties"], b["properties"], ignore_list_order)
      elsif (a.keys.sort == ["destination", "labels", "properties", "source", "type"] || a.keys.sort == ["destination", "key", "labels", "properties", "source", "type"]) && (a["type"] == "path") && (b["type"] == "path")
        return a["labels"].sort == b["labels"].sort && compare(a["properties"], b["properties"], ignore_list_order) && compare(a["source"], b["source"], ignore_list_order) && compare(a["destination"], b["destination"], ignore_list_order)
      else
        return false unless a.keys.sort == b.keys.sort
        a.keys.each do |k|
          return false unless compare(a[k], b[k], ignore_list_order)
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
    elsif a.class == Float
      case b
      when String
        return a.nan? && b == "NaN"
      else
        return false
      end
    else
      return false
    end
  end
end

def compare_table_in_any_order(actual, expected, ignore_list_order)
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
      if compare(actual[i], expected[j], ignore_list_order)
        match = true
        break
      end
    end
    if !match
      return false
    end
  end
  return true
end

def compare_table_in_order(actual, expected, ignore_list_order)
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
    unless compare(actual[i], expected[i], ignore_list_order)
      puts("actual '#{actual[i]}' != '#{expected[i]}'")
      return false
    end
  end
  return true
end

RSpec::Matchers.matcher :eq_in_any_order do |expected|
  match do |actual|
    compare_table_in_any_order(actual, expected, false)
  end
end

RSpec::Matchers.matcher :eq_in_any_order_ignoring_lists_order do |expected|
  match do |actual|
    compare_table_in_any_order(actual, expected, true)
  end
end

RSpec::Matchers.matcher :eq_in_order do |expected|
  match do |actual|
    compare_table_in_order(actual, expected, false)
  end
end

RSpec::Matchers.matcher :eq_in_order_ignoring_lists_order do |expected|
  match do |actual|
    compare_table_in_order(actual, expected, true)
  end
end