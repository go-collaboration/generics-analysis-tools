require "compiler/crystal/syntax"
require "compiler/requires"

class GenericClassCounter < Crystal::Visitor
  getter class_ty_vars = 0
  getter mod_ty_vars = 0
  getter class_no_ty_vars = 0
  getter mod_no_ty_vars = 0

  def visit(node : Crystal::ClassDef)
    tyvars = node.type_vars
    if tyvars
      if !tyvars.empty?
        @class_ty_vars += 1
      else
        @class_no_ty_vars += 1
      end
    else
        @class_no_ty_vars += 1
    end

    node.body.try &.accept(self)
  end

  def visit(node : Crystal::ModuleDef)
    tyvars = node.type_vars
    if tyvars
      if !tyvars.empty?
        @mod_ty_vars += 1
      else
        @mod_no_ty_vars += 1
      end
    else
      @mod_no_ty_vars += 1
    end

    node.body.try &.accept(self)
  end

  def visit(node : Crystal::ASTNode)
    node.accept_children(self)
  end
end

counter = GenericClassCounter.new
parse_errors = 0

ARGV.each do |path|
  code = File.read(path)
  begin
    parser = Crystal::Parser.new(code)
    parser.filename = path
    node = parser.parse
  rescue
    parse_errors += 1
    STDERR.puts "unable to parse #{path}"
    next
  end

  node.accept(counter)
end

# puts "Generic classes: #{counter.class_ty_vars}"
# puts "Non-Generic classes: #{counter.class_no_ty_vars}"
# puts "Generic modules: #{counter.mod_ty_vars}"
# puts "Non-Generic modules: #{counter.mod_no_ty_vars}"
puts "#{parse_errors},#{counter.class_ty_vars},#{counter.class_no_ty_vars},0,0"
