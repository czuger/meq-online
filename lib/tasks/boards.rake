namespace :board do

  desc 'Delete all boards'
  task delete_all: :environment do

    Board.all.each do |board|
      board.users.clear
      board.destroy
    end

  end
end


