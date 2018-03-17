acc = []

File.open("numbers") do |f|
  f.each_line { |line| acc << line.to_i }
end

puts acc.sort!.first
