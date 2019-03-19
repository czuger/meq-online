class BoardMessagesController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board
  before_action :set_board_message, only: [:show, :edit, :update, :destroy]

  # GET /board_messages
  # GET /board_messages.json
  def index
    @board_messages = BoardMessage.where(reciever_id: @actor)
  end

  # GET /board_messages/1
  # GET /board_messages/1.json
  # def show
  # end

  # GET /board_messages/new
  def new
    @board_message = BoardMessage.new

    @heroes = GameData::Heroes.new

    @recievers = []

    @recievers << [ "Sauron (#{@board.sauron.user.name})", @board.sauron.id ] unless @actor == @board.sauron

    @board.heroes.includes(:user).each do |h|
      next if @actor == h
      @heroes_hero = @heroes.get( h.name_code )
      @recievers << [ "#{@heroes_hero.name} (#{h.user.name})", h.id ]
    end
  end

  # GET /board_messages/1/edit
  # def edit
  # end

  # POST /board_messages
  # POST /board_messages.json
  def create
    @board_message = BoardMessage.new(board_message_params)

    respond_to do |format|
      if @board_message.save
        format.html { redirect_to @board_message, notice: 'Board message was successfully created.' }
        format.json { render :show, status: :created, location: @board_message }
      else
        format.html { render :new }
        format.json { render json: @board_message.errors, status: :unprocessable_entity }
      end
    end
  end

  # # PATCH/PUT /board_messages/1
  # # PATCH/PUT /board_messages/1.json
  # def update
  #   respond_to do |format|
  #     if @board_message.update(board_message_params)
  #       format.html { redirect_to @board_message, notice: 'Board message was successfully updated.' }
  #       format.json { render :show, status: :ok, location: @board_message }
  #     else
  #       format.html { render :edit }
  #       format.json { render json: @board_message.errors, status: :unprocessable_entity }
  #     end
  #   end
  # end
  #
  # # DELETE /board_messages/1
  # # DELETE /board_messages/1.json
  # def destroy
  #   @board_message.destroy
  #   respond_to do |format|
  #     format.html { redirect_to board_messages_url, notice: 'Board message was successfully destroyed.' }
  #     format.json { head :no_content }
  #   end
  # end

  private
    # Use callbacks to share common setup or constraints between actions.
    def set_board_message
      @board_message = BoardMessage.find(params[:id])
    end

    # Never trust parameters from the scary internet, only allow the white list through.
    def board_message_params
      params.require(:board_message).permit(:board_id, :sender, :reciever, :text)
    end
end
