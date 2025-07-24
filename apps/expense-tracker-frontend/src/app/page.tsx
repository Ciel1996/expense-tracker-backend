import { healthCheck, currentUser } from '';

export default async function Index() {
  let healthStatus = '';
  let userName = '';

  try {
    const response = await healthCheck();
    console.log(response.data);
    healthStatus = response.data;
  } catch (error) {
    console.log(error);
    healthStatus = "Error fetching health status";
  }

  try {
    // TODO: get token an insert it
    const usernameReponse = await currentUser();
    console.log(usernameReponse.status);
    userName = usernameReponse.data.name;
  } catch (error) {
    console.log(error);
    userName = "Anonymous"
  }

  /*
   * Replace the elements below with your own.
   *
   * Note: The corresponding styles are in the ./index.tailwind file.
   */
  return (
    <div>
      <div className="wrapper">
        <div className="container">
          <div id="welcome">
            <h1>
              <span> Hello {userName}, </span>
              Welcome to expense-tracker-frontend 👋

              <span>Message from ExpenseTracker: {healthStatus} </span>
            </h1>
          </div>
        </div>
      </div>
    </div>
  );
}
