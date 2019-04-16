namespace :board do

  desc 'Delete all boards'
  task delete_all: :environment do

    Board.all.each do |board|
      board.users.clear
      board.logs.delete_all
      # board.combat&.destroy

      board.current_hero = nil
      board.aasm_state = :created
      board.save!

      board.heroes.each{ |e| MovementPreparationStep.where( actor_id: e.id ).delete_all }

      board.combat.destroy if board.combat

      board.heroes.destroy_all
      board.destroy
    end

  end
end


