// SPDX-License-Identifier: LicenseRef-VPL WITH AGPL-3.0-only
pragma solidity ^0.8.25;

import { ERC721A } from "ERC721A/ERC721A.sol";
import { Ownable } from "solady/auth/Ownable.sol";
import { ReentrancyGuard } from "soledge/utils/ReentrancyGuard.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title Soup for Shiro
  @author Tim Clancy <tim-clancy.eth>
  @custom:terry "It's good for the soul."
  @custom:preserve

  Get well soon; Ethereum needs you! <3

  Ultra-Satisfying Chicken Noodle Soup from Joanne Gallagher
  PREP 5min
  COOK 35min
  TOTAL 40min

  You will love this homemade chicken noodle soup, which skips cooking a whole
  chicken and calls on boneless, skinless chicken thighs instead. The soup
  tastes incredible, satisfying, and classic, and the significantly reduced
  cooking time makes it the best chicken noodle soup for a weeknight.

  6 Servings

  You Will Need
  - 1 pound skinless, boneless chicken thighs, 4 to 5 thighs
  - 5 ounces egg noodles or pasta of choice
  - 2 tablespoons butter, chicken fat, or olive oil
  - 1 large onion, chopped
  - 2 large carrots, chopped
  - 2 ribs celery, chopped, optional
  - 1 heaped tablespoon minced garlic, 4 cloves
  - 2 bay leaves
  - 3 sprigs fresh thyme or use 1/2 teaspoon dried thyme
  - 8 cups chicken stock or broth, low sodium, or use homemade stock
  - Salt and pepper, to taste
  - 1/4 cup fresh parsley, finely chopped
  - Water or more stock, as needed

  Directions

  Melt butter in a large pot or Dutch oven over medium heat. Add the onions,
  carrots, and celery. Cook, stirring every few minutes until the vegetables
  begin to soften; 5 to 6 minutes.

  Stir in the garlic, bay leaves, and thyme. Cook, while stirring the garlic
  around the pan, for about 1 minute.

  Pour in the chicken stock and bring to a low simmer. Taste the soup then
  adjust the seasoning with salt and pepper. Depending on the stock used, you
  might need to add 1 or more teaspoons of salt.

  Submerge the chicken thighs into the soup so that the broth covers them. Bring
  the soup back to a low simmer then partially cover the pot with a lid and
  cook, stirring a few times until the chicken thighs are cooked through; about
  20 minutes.

  If, during this time, the broth seems low, add a splash more stock or a bit of
  water. Turn the heat to medium-low.

  Transfer the cooked chicken to a plate. Stir the noodles into the soup and
  cook until done, 6 to 10 minutes depending on the type of noodles used.

  While the noodles cook, shred the chicken into strips or dice into cubes.
  Slide the chicken back into the pot and then taste the soup once more for
  seasoning. Adjust with more salt and pepper, as needed. Stir in the parsley
  and serve.

  Tips

  Refrigerate in an airtight container for 3 to 4 days. Or freeze for up to 3
  months. As the soup sits, the noodles soak up the broth. When you reheat, add
  a splash of extra broth or water.

  Make ahead: Follow the recipe above, but do not add the noodles. Refrigerate
  or freeze the soup. When ready to reheat, bring the soup to a low simmer and
  add the dried noodles. Cook until they are done, and enjoy.

  Seasoning the Soup: If you feel the soup is missing some zing, add a bit more
  salt. You can also add a pop of flavor with a squeeze of fresh lemon juice, a
  dash of fish sauce (we use this trick for store-bought stocks and broths
  often) or Worcestershire sauce.

  Using rotisserie chicken: Add two to three cups of shredded or diced cooked
  chicken to the soup when you add the dried noodles and reduce the simmer time
  by 10 minutes. (I love using leftover roasted chicken!)

  NUTRITION: Calories 298 / Total Fat 11.4g / Saturated Fat 4.4g
  / Cholesterol 97.7mg / Sodium 748.7mg / Carbohydrate 22.9g
  / Dietary Fiber 1.7g / Total Sugars 7.5g / Protein 24.8g

  @custom:date June 23rd, 2025.
*/
contract Soup is
  ERC721A,
  Ownable,
  ReentrancyGuard {

  /**
    Construct a new instance of Soup.

    @param _owner The initial owner of Soup.
  */
  constructor (
    address _owner
  ) ERC721A("Soup!", "SOUP") {
    _initializeOwner(_owner);
  }

  /**
    Returns the same URI for every token.

    @return _ The URI for every token.
  */
  function tokenURI (
    uint256
  ) public pure override returns (string memory) {
    return
    "data:application/json;base64,eyJuYW1lIjoiU291cCBmb3IgU2hpcm8hIiwiZXh0ZXJuYWxfdXJsIjoiaHR0cHM6Ly9ldGhlcmV1bS5vcmciLCJpbWFnZSI6ImRhdGE6aW1hZ2Uvc3ZnK3htbDtiYXNlNjQsUEhOMlp5QjNhV1IwYUQwaU1qVTJJaUJvWldsbmFIUTlJakkxTmlJZ2RtbGxkMEp2ZUQwaU1qQWdMVE15SURJMU5pQXlOVFlpSUhodGJHNXpQU0pvZEhSd09pOHZkM2QzTG5jekxtOXlaeTh5TURBd0wzTjJaeUkrUEhCaGRHZ2daRDBpVFRZd0lERXdNQ0JCT1RBZ01qVWdNQ0F3SURBZ01qUXdJREV3TUNCUk1qTTFJREUxTUNBeE5UQWdNVFkySUZFMk5TQXhOVEFnTmpBZ01UQXdJRm9pSUdacGJHdzlJaU00WWpWbE0yTWlJSE4wY205clpUMGlJelZqTTJFeU1TSWdjM1J5YjJ0bExYZHBaSFJvUFNJeUlpOCtQR1ZzYkdsd2MyVWdZM2c5SWpFMU1DSWdZM2s5SWpFd01DSWdjbmc5SWprd0lpQnllVDBpTWpVaUlHWnBiR3c5SWlOa05tRXhNMk1pTHo0OFpXeHNhWEJ6WlNCamVEMGlNVFV3SWlCamVUMGlNVEF3SWlCeWVEMGlPVEFpSUhKNVBTSXlOU0lnWm1sc2JEMGlibTl1WlNJZ2MzUnliMnRsUFNJak5XTXpZVEl4SWlCemRISnZhMlV0ZDJsa2RHZzlJaklpTHo0OFkybHlZMnhsSUdONFBTSXhNVFVpSUdONVBTSTVOU0lnY2owaU5DSWdabWxzYkQwaUkyRTFNbUV5WVNJdlBqeGphWEpqYkdVZ1kzZzlJakUzTlNJZ1kzazlJakV3TlNJZ2NqMGlOQ0lnWm1sc2JEMGlJMkUxTW1FeVlTSXZQanhqYVhKamJHVWdZM2c5SWpFek5TSWdZM2s5SWprd0lpQnlQU0kwSWlCbWFXeHNQU0lqTWpJNFlqSXlJaTgrUEhKbFkzUWdlRDBpTVRrd0lpQjVQU0k1TlNJZ2QybGtkR2c5SWpVaUlHaGxhV2RvZEQwaU5TSWdabWxzYkQwaUkyWm1abVptWmlJZ2NuZzlJakVpTHo0OGNtVmpkQ0I0UFNJeE1qQWlJSGs5SWpFd09DSWdkMmxrZEdnOUlqVWlJR2hsYVdkb2REMGlOU0lnWm1sc2JEMGlJMlptWm1abVppSWdjbmc5SWpFaUx6NDhZMmx5WTJ4bElHTjRQU0l4TlRVaUlHTjVQU0k1TlNJZ2NqMGlNeUlnWm1sc2JEMGlJMlptWVRBM1lTSXZQanhqYVhKamJHVWdZM2c5SWpFME1DSWdZM2s5SWpFeE1DSWdjajBpTXlJZ1ptbHNiRDBpSTJKa1lqYzJZaUl2UGp4bGJHeHBjSE5sSUdONFBTSXhOalVpSUdONVBTSTVNQ0lnY25nOUlqTWlJSEo1UFNJeUlpQm1hV3hzUFNJak5tSTRaVEl6SWk4K1BHTnBjbU5zWlNCamVEMGlNVE13SWlCamVUMGlNVEF3SWlCeVBTSXlJaUJtYVd4c1BTSWpNREEyTkRBd0lpOCtQSEJoZEdnZ1pEMGlUVEV6TUNBMU1DQkRNVEkxSURRd0xDQXhNelVnTXpBc0lERXpNQ0F5TUNJZ2MzUnliMnRsUFNJalpHUmtJaUJ6ZEhKdmEyVXRkMmxrZEdnOUlqSWlJR1pwYkd3OUltNXZibVVpTHo0OGNHRjBhQ0JrUFNKTk1UVXdJRFV3SUVNeE5EVWdOREFzSURFMU5TQXpNQ3dnTVRVd0lESXdJaUJ6ZEhKdmEyVTlJaU5rWkdRaUlITjBjbTlyWlMxM2FXUjBhRDBpTWlJZ1ptbHNiRDBpYm05dVpTSXZQanh3WVhSb0lHUTlJazB4TnpBZ05UQWdRekUyTlNBME1Dd2dNVGMxSURNd0xDQXhOekFnTWpBaUlITjBjbTlyWlQwaUkyUmtaQ0lnYzNSeWIydGxMWGRwWkhSb1BTSXlJaUJtYVd4c1BTSnViMjVsSWk4K1BDOXpkbWMrIn0=";
  }

  /**
    This function allows the owner to mint soup.

    @param _destination An address to receive the soup.
    @param _amount The amount to mint.
  */
  function mint (
    address _destination,
    uint96 _amount
  ) external payable nonReentrant onlyOwner {
    _mint(_destination, _amount);
  }
}
