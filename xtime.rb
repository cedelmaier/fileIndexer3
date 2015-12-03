#!/usr/bin/env ruby
def mem(pid); `ps p #{pid} -o rss`.split.last.to_i; end
def nlwp(pid); `ps p #{pid} -o nlwp`.split.last.to_i; end
t = Time.now
pid = Process.spawn(*ARGV.to_a)
mm = 0
pp = 0

Thread.new do
  mm = mem(pid)
  pp = nlwp(pid)
  while true
    sleep 0.3
    m = mem(pid)
    p = nlwp(pid)
    mm = m if m > mm
    pp = p if p > pp
  end
end

Process.waitall
STDERR.puts "%.2fs, %.1fMb, %dnlwp" % [Time.now - t, mm / 1024.0, pp]

